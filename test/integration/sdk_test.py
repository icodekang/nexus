#!/usr/bin/env python3
"""
Nexus SDK Integration Test
===========================
Tests OpenAI SDK and Anthropic SDK accessing LLMs through Nexus API key.

Prerequisites:
  1. Nexus server running at NEXUS_BASE_URL (default: http://localhost:8080)
  2. At least one provider API key configured (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
  3. At least one model registered in the database
  4. Python 3.8+ with openai and anthropic packages installed

Usage:
  # Install dependencies
  pip install -r test/integration/requirements.txt

  # Run with defaults
  python test/integration/sdk_test.py

  # Custom Nexus server
  NEXUS_BASE_URL=http://localhost:8080 python test/integration/sdk_test.py

  # Custom test credentials
  NEXUS_BASE_URL=http://localhost:8080 \
  NEXUS_TEST_EMAIL=admin@nexus.dev \
  NEXUS_TEST_PASSWORD=admin123 \
  python test/integration/sdk_test.py
"""

import os
import sys
import json
import uuid
import argparse
import urllib.request
import urllib.error
import ssl
from typing import Optional, Dict, Any


# --- Configuration ---

NEXUS_BASE_URL = os.environ.get("NEXUS_BASE_URL", "http://localhost:8080")
NEXUS_V1_URL = f"{NEXUS_BASE_URL.rstrip('/')}/v1"
TEST_EMAIL = os.environ.get("NEXUS_TEST_EMAIL", "admin@nexus.dev")
TEST_PASSWORD = os.environ.get("NEXUS_TEST_PASSWORD", "admin123")
NEXUS_API_KEY = os.environ.get("NEXUS_API_KEY", "")

# Allow self-signed certs for local testing
SSL_CONTEXT = ssl.create_default_context()
SSL_CONTEXT.check_hostname = False
SSL_CONTEXT.verify_mode = ssl.CERT_NONE


# --- Test Framework ---

_tests = []
_failures = 0
_skipped = []


def test(name: str):
    """Decorator to register a test function."""
    def decorator(fn):
        _tests.append((name, fn))
        return fn
    return decorator


class TestSkip(Exception):
    pass


class TestFail(Exception):
    pass


def skip(reason: str):
    """Signal that the current test should be skipped."""
    _skipped.append(reason)
    raise TestSkip(reason)


def expect(actual, msg: str = ""):
    """Assertion helper returning a matcher."""
    prefix = f"{msg}: " if msg else ""
    return _Expect(actual, prefix)


class _Expect:
    def __init__(self, actual, prefix: str):
        self.actual = actual
        self.prefix = prefix

    def to_equal(self, expected):
        if self.actual != expected:
            raise TestFail(f"{self.prefix}expected {expected!r}, got {self.actual!r}")

    def to_contain(self, substr):
        if substr not in str(self.actual):
            raise TestFail(f"{self.prefix}{self.actual!r} does not contain {substr!r}")

    def to_be_truthy(self):
        if not self.actual:
            raise TestFail(f"{self.prefix}expected truthy, got {self.actual!r}")

    def to_be_instance(self, cls):
        if not isinstance(self.actual, cls):
            raise TestFail(
                f"{self.prefix}expected instance of {cls.__name__}, got {type(self.actual).__name__}"
            )

    def to_be_greater_than(self, n):
        if self.actual <= n:
            raise TestFail(f"{self.prefix}expected > {n}, got {self.actual}")


# --- HTTP Helpers ---

def _request(
    method: str,
    path: str,
    body: Optional[Dict[str, Any]] = None,
    headers: Optional[Dict[str, str]] = None,
) -> Dict[str, Any]:
    """Make an HTTP request to the Nexus server."""
    url = f"{NEXUS_BASE_URL.rstrip('/')}{path}"
    data = json.dumps(body).encode("utf-8") if body else None
    req_headers = {"Content-Type": "application/json"}
    if headers:
        req_headers.update(headers)

    req = urllib.request.Request(url, data=data, headers=req_headers, method=method)
    try:
        with urllib.request.urlopen(req, context=SSL_CONTEXT, timeout=30) as resp:
            content = resp.read().decode("utf-8")
            if not content.strip():
                return {}
            return json.loads(content)
    except urllib.error.HTTPError as e:
        error_body = e.read().decode("utf-8")
        raise RuntimeError(
            f"HTTP {e.code} on {method} {path}: {error_body}"
        ) from e


def _auth_header(token: str) -> Dict[str, str]:
    return {"Authorization": f"Bearer {token}"}


# --- Setup / Teardown ---

class TestContext:
    """Holds shared state across tests."""
    jwt_token: str = ""
    api_key: str = ""
    api_key_id: str = ""
    available_model: str = ""
    available_provider: str = ""


ctx = TestContext()


def setup():
    """One-time setup: login and discover available models."""
    print("\n" + "=" * 60)
    print("  Nexus SDK Integration Test Setup")
    print("=" * 60)

    if NEXUS_API_KEY:
        print(f"\n  Using existing API key: {NEXUS_API_KEY[:20]}...")
        ctx.api_key = NEXUS_API_KEY
    else:
        # 1. Login
        print(f"\n  Logging in as {TEST_EMAIL} ...")
        resp = _request("POST", "/v1/auth/login", {
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
        })
        ctx.jwt_token = resp["token"]
        expect(ctx.jwt_token, "JWT token").to_be_truthy()
        print(f"  [OK] Logged in, token: {ctx.jwt_token[:20]}...")

    # 2. Discover models
    print("\n  Fetching available models ...")
    headers = _auth_header(ctx.api_key) if NEXUS_API_KEY else _auth_header(ctx.jwt_token)
    resp = _request("GET", "/v1/models", headers=headers)
    models = resp.get("data", [])
    print(f"  Found {len(models)} model(s)")

    if not models:
        print("  [WARN] No models configured in Nexus database!")
        print("  [WARN] SDK integration tests will be skipped.")
        print("  [WARN] Register a model via admin dashboard or database first.")
        return

    for m in models:
        print(f"    - {m.get('id')} ({m.get('provider', 'unknown')}) - {m.get('name', 'N/A')}")

    ctx.available_model = models[0].get("id", "")
    ctx.available_provider = models[0].get("provider", "")
    print(f"  Using model: {ctx.available_model} (provider: {ctx.available_provider})")


# --- SDK Imports (lazy) ---

_openai_client = None
_anthropic_client = None


def _get_openai_client():
    global _openai_client
    if _openai_client is None:
        try:
            from openai import OpenAI
        except ImportError:
            skip("openai package not installed. Run: pip install openai")
        _openai_client = OpenAI(
            base_url=NEXUS_V1_URL + "/openai",
            api_key=ctx.api_key,
        )
    return _openai_client


def _get_anthropic_client():
    global _anthropic_client
    if _anthropic_client is None:
        try:
            from anthropic import Anthropic
        except ImportError:
            skip("anthropic package not installed. Run: pip install anthropic")
        _anthropic_client = Anthropic(
            base_url=NEXUS_BASE_URL,
            api_key=ctx.api_key,
        )
    return _anthropic_client


# --- Test Cases ---


# === API Key Management ===

@test("Create Nexus API Key")
def test_create_api_key():
    if NEXUS_API_KEY:
        skip("Using existing NEXUS_API_KEY, key creation skipped")

    resp = _request("POST", "/v1/me/keys", {
        "name": f"sdk-test-{uuid.uuid4().hex[:8]}",
    }, headers=_auth_header(ctx.jwt_token))

    ctx.api_key = resp["key"]
    ctx.api_key_id = resp["id"]

    expect(ctx.api_key, "api key").to_contain("sk-nexus-")
    expect(resp["id"], "key id").to_be_truthy()
    print(f"  Created key: {ctx.api_key[:20]}...")


@test("List API Keys includes created key")
def test_list_api_keys():
    if NEXUS_API_KEY:
        skip("Using existing NEXUS_API_KEY, list check skipped")

    resp = _request("GET", "/v1/me/keys", headers=_auth_header(ctx.jwt_token))
    data = resp.get("data", [])
    key_ids = [k["id"] for k in data]
    expect(ctx.api_key_id in key_ids, f"key {ctx.api_key_id} in list").to_be_truthy()


@test("Nexus API Key is valid for model list")
def test_api_key_model_list():
    resp = _request("GET", "/v1/models", headers=_auth_header(ctx.api_key))
    data = resp.get("data", [])
    expect(len(data) > 0, "models list non-empty").to_be_truthy()


# === OpenAI SDK ===

@test("OpenAI SDK: non-streaming chat completion")
def test_openai_sdk_non_streaming():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_openai_client()
    response = client.chat.completions.create(
        model=ctx.available_model,
        messages=[
            {"role": "user", "content": "Say 'hello' in exactly one word."}
        ],
        max_tokens=50,
        temperature=0.0,
        stream=False,
    )

    expect(response.choices, "choices").to_be_truthy()
    expect(len(response.choices), "at least one choice").to_be_greater_than(0)

    content = response.choices[0].message.content
    expect(content, "response content").to_be_truthy()
    print(f"  Response: {content[:200]}")

    if response.usage:
        expect(response.usage.prompt_tokens, "prompt tokens").to_be_greater_than(0)
        expect(response.usage.completion_tokens, "completion tokens").to_be_greater_than(0)
        print(f"  Usage: prompt={response.usage.prompt_tokens} "
              f"completion={response.usage.completion_tokens}")


@test("OpenAI SDK: streaming chat completion")
def test_openai_sdk_streaming():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_openai_client()
    stream = client.chat.completions.create(
        model=ctx.available_model,
        messages=[
            {"role": "user", "content": "Count from 1 to 3, one number per line."}
        ],
        max_tokens=50,
        temperature=0.0,
        stream=True,
    )

    chunks = []
    for chunk in stream:
        chunks.append(chunk)

    expect(len(chunks) > 0, "received at least one chunk").to_be_truthy()

    full_text = ""
    for chunk in chunks:
        delta = chunk.choices[0].delta
        if delta.content:
            full_text += delta.content

    expect(full_text, "accumulated text").to_be_truthy()
    print(f"  Full streamed text: {full_text[:200]}")


@test("OpenAI SDK: system message and multi-turn conversation")
def test_openai_sdk_multi_turn():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_openai_client()
    response = client.chat.completions.create(
        model=ctx.available_model,
        messages=[
            {"role": "system", "content": "You are a helpful math assistant. Reply with ONLY the numerical answer, no explanation."},
            {"role": "user", "content": "What is 2 + 2?"},
            {"role": "assistant", "content": "4"},
            {"role": "user", "content": "Now multiply that by 3."},
        ],
        max_tokens=50,
        temperature=0.0,
        stream=False,
    )

    content = response.choices[0].message.content.strip()
    expect(content, "multi-turn response").to_be_truthy()
    print(f"  Multi-turn response: {content[:200]}")


@test("OpenAI SDK: streaming with stop reason")
def test_openai_sdk_streaming_stop_reason():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_openai_client()
    stream = client.chat.completions.create(
        model=ctx.available_model,
        messages=[
            {"role": "user", "content": "Say just 'Hello World' and nothing else."}
        ],
        max_tokens=20,
        temperature=0.0,
        stream=True,
    )

    last_finish_reason = None
    for chunk in stream:
        choice = chunk.choices[0]
        if choice.finish_reason:
            last_finish_reason = choice.finish_reason

    expect(last_finish_reason, "finish_reason").to_be_truthy()
    print(f"  Finish reason: {last_finish_reason}")


# === Anthropic SDK ===

@test("Anthropic SDK: non-streaming messages")
def test_anthropic_sdk_non_streaming():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_anthropic_client()
    response = client.messages.create(
        model=ctx.available_model,
        max_tokens=100,
        messages=[
            {"role": "user", "content": "Say 'hello' in exactly one word."}
        ],
        temperature=0.0,
    )

    expect(response, "response").to_be_truthy()
    expect(response.id, "message id").to_be_truthy()
    expect(response.type, "type").to_equal("message")
    expect(response.content, "content").to_be_truthy()
    expect(len(response.content), "content blocks").to_be_greater_than(0)

    text = response.content[0].text
    expect(text, "response text").to_be_truthy()
    print(f"  Response: {text[:200]}")

    expect(response.model, "model").to_equal(ctx.available_model)
    expect(response.stop_reason, "stop_reason").to_be_truthy()

    if response.usage:
        expect(response.usage.input_tokens, "input tokens").to_be_greater_than(0)
        print(f"  Usage: input={response.usage.input_tokens} "
              f"output={response.usage.output_tokens}")


@test("Anthropic SDK: streaming messages")
def test_anthropic_sdk_streaming():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_anthropic_client()
    stream = client.messages.create(
        model=ctx.available_model,
        max_tokens=50,
        messages=[
            {"role": "user", "content": "Count from 1 to 3, one number per line."}
        ],
        temperature=0.0,
        stream=True,
    )

    events = []
    accumulated_text = ""
    for event in stream:
        events.append(event)
        if event.type == "content_block_delta":
            accumulated_text += event.delta.text

    expect(len(events) > 0, "received events").to_be_truthy()
    expect(accumulated_text, "accumulated text").to_be_truthy()
    print(f"  Full streamed text: {accumulated_text[:200]}")

    event_types = [e.type for e in events]
    expect("message_start" in event_types, "has message_start").to_be_truthy()
    expect("content_block_start" in event_types, "has content_block_start").to_be_truthy()
    expect("content_block_delta" in event_types, "has content_block_delta").to_be_truthy()
    expect("message_stop" in event_types, "has message_stop").to_be_truthy()


@test("Anthropic SDK: system prompt")
def test_anthropic_sdk_system():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_anthropic_client()
    response = client.messages.create(
        model=ctx.available_model,
        max_tokens=50,
        system="You are a math expert. Reply with ONLY the numerical answer.",
        messages=[
            {"role": "user", "content": "What is 7 * 8?"}
        ],
        temperature=0.0,
    )

    text = response.content[0].text
    expect(text, "system-prompted response").to_be_truthy()
    print(f"  Response: {text[:200]}")


@test("Anthropic SDK: multi-turn conversation")
def test_anthropic_sdk_multi_turn():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_anthropic_client()
    response = client.messages.create(
        model=ctx.available_model,
        max_tokens=50,
        messages=[
            {"role": "user", "content": "The capital of France is Paris."},
            {"role": "assistant", "content": "Yes, that's correct."},
            {"role": "user", "content": "What is the capital of Germany?"},
        ],
        temperature=0.0,
    )

    text = response.content[0].text
    expect(text, "multi-turn response").to_be_truthy()
    print(f"  Multi-turn response: {text[:200]}")


@test("Anthropic SDK: response structure validation")
def test_anthropic_sdk_response_structure():
    if not ctx.available_model:
        skip("No model configured")

    client = _get_anthropic_client()
    response = client.messages.create(
        model=ctx.available_model,
        max_tokens=50,
        messages=[{"role": "user", "content": "Say hi"}],
        temperature=0.0,
    )

    expect(response.id, "id").to_contain("msg_")
    expect(response.type, "type").to_equal("message")
    expect(response.role, "role").to_equal("assistant")
    expect(response.model, "model").to_be_truthy()
    expect(response.stop_reason, "stop_reason").to_be_truthy()
    expect(response.content, "content").to_be_instance(list)
    expect(len(response.content), "content count").to_be_greater_than(0)
    expect(response.content[0].type, "content type").to_equal("text")
    expect(response.content[0].text, "content text").to_be_truthy()
    print(f"  All response fields validated: id={response.id}, model={response.model}")


# === Cross-SDK Tests ===

@test("Same model accessible via both OpenAI and Anthropic SDKs")
def test_cross_sdk_same_model():
    if not ctx.available_model:
        skip("No model configured")

    question = "What is 1+1? Reply with just the number."

    oai_client = _get_openai_client()
    oai_resp = oai_client.chat.completions.create(
        model=ctx.available_model,
        messages=[{"role": "user", "content": question}],
        max_tokens=50,
        temperature=0.0,
        stream=False,
    )
    oai_text = oai_resp.choices[0].message.content.strip()

    client = _get_anthropic_client()
    ant_resp = client.messages.create(
        model=ctx.available_model,
        max_tokens=50,
        messages=[{"role": "user", "content": question}],
        temperature=0.0,
    )
    ant_text = ant_resp.content[0].text.strip()

    expect(oai_text, "OpenAI response").to_be_truthy()
    expect(ant_text, "Anthropic response").to_be_truthy()
    print(f"  OpenAI SDK response:    {oai_text[:100]}")
    print(f"  Anthropic SDK response: {ant_text[:100]}")


# --- Cleanup ---

def teardown():
    """Delete the API key created during tests."""
    print("\n" + "=" * 60)
    print("  Cleanup")
    print("=" * 60)
    if NEXUS_API_KEY:
        print("  Using existing NEXUS_API_KEY, no cleanup needed.")
        return
    if ctx.api_key_id:
        try:
            _request(
                "DELETE",
                f"/v1/me/keys/{ctx.api_key_id}",
                headers=_auth_header(ctx.jwt_token),
            )
            print(f"  [OK] Deleted API key: {ctx.api_key_id}")
        except Exception as e:
            print(f"  [WARN] Failed to delete key: {e}")
    else:
        print("  No API key to clean up.")


# --- Runner ---

def main():
    global NEXUS_BASE_URL, TEST_EMAIL, TEST_PASSWORD, NEXUS_API_KEY

    parser = argparse.ArgumentParser(
        description="Nexus SDK Integration Test - test OpenAI/Anthropic SDKs via Nexus API key"
    )
    parser.add_argument(
        "--base-url",
        default=NEXUS_BASE_URL,
        help=f"Nexus server base URL (default: {NEXUS_BASE_URL})",
    )
    parser.add_argument(
        "--api-key",
        default=NEXUS_API_KEY,
        help="Existing Nexus API key (sk-nexus-...) — skips login and key creation",
    )
    parser.add_argument(
        "--email",
        default=TEST_EMAIL,
        help=f"Test user email (default: {TEST_EMAIL})",
    )
    parser.add_argument(
        "--password",
        default=TEST_PASSWORD,
        help="Test user password",
    )
    parser.add_argument(
        "-k", "--filter",
        default="",
        help="Run only tests whose name contains the given substring",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List available tests without running",
    )
    args = parser.parse_args()

    NEXUS_BASE_URL = args.base_url
    NEXUS_API_KEY = args.api_key
    TEST_EMAIL = args.email
    TEST_PASSWORD = args.password

    if args.list:
        print("Available tests:")
        for i, (name, _) in enumerate(_tests, 1):
            print(f"  {i:2d}. {name}")
        return 0

    # Setup
    try:
        setup()
    except Exception as e:
        print(f"\n  [FATAL] Setup failed: {e}")
        print("  Make sure the Nexus server is running and reachable.")
        return 1

    # Run tests
    print("\n" + "=" * 60)
    print("  Running Tests")
    print("=" * 60)

    passed = 0
    failed = 0
    skipped_count = 0

    for name, fn in _tests:
        if args.filter and args.filter.lower() not in name.lower():
            continue

        print(f"\n  [{name}]")
        _skipped.clear()
        try:
            fn()
            passed += 1
            print(f"  [PASS] {name}")
        except TestSkip as e:
            skipped_count += 1
            print(f"  [SKIP] {name} - {e}")
        except Exception as e:
            failed += 1
            print(f"  [FAIL] {name}: {e}")

    # Teardown
    teardown()

    # Summary
    print("\n" + "=" * 60)
    print("  Results")
    print("=" * 60)
    print(f"  Passed:  {passed}")
    print(f"  Failed:  {failed}")
    print(f"  Skipped: {skipped_count}")
    print(f"  Total:   {passed + failed + skipped_count}")
    print("=" * 60)

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
