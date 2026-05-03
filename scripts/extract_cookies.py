#!/usr/bin/env python3
"""
Nexus Cookie Extractor
弹出内嵌浏览器，手动登录后自动获取 Cookie

Usage:
    python3 scripts/extract_cookies.py --provider claude
    python3 scripts/extract_cookies.py --provider chatgpt
"""

import argparse
import json
import sys
import time
from dataclasses import dataclass
from typing import Optional

try:
    from playwright.sync_api import sync_playwright
except ImportError:
    print("Error: playwright not installed. Run: pip install playwright && playwright install chromium")
    sys.exit(1)


@dataclass
class PersistedSession:
    cookies: dict
    auth_tokens: dict
    expires_at: Optional[str] = None

    def to_json(self) -> str:
        return json.dumps({"cookies": self.cookies, "auth_tokens": self.auth_tokens, "expires_at": self.expires_at})


PROVIDER_URLS = {
    "claude": "https://claude.ai/login",
    "chatgpt": "https://chat.openai.com/login",
    "deepseek": "https://chat.deepseek.com/",
    "google": "https://ai.google.dev",
}

LOGIN_SUCCESS_PATTERNS = {
    "claude": ["claude.ai/chat", "claude.ai/project"],
    "chatgpt": ["chat.openai.com/chat", "chatgpt.com/chat"],
    "deepseek": ["chat.deepseek.com/"],  # root path (or with query params) means logged in
    "google": ["ai.google.dev/app", "ai.google.dev/studio"],
}

# URL patterns that indicate the user is NOT logged in yet
LOGIN_FAILURE_PATTERNS = {
    "claude": ["/login", "/sign_in", "/signin", "/auth/login"],
    "chatgpt": ["/login", "/sign_in", "/signin", "/auth/login"],
    "deepseek": ["/sign_in", "/signin", "/login"],
    "google": ["/signin", "/login"],
}

# Page content patterns that indicate a LOGIN FORM (not just any login mention)
# These are full phrases only found on actual login/register forms
LOGIN_FORM_INDICATORS = [
    # Phone verification form (DeepSeek)
    "发送验证码", "输入手机号",
    # Generic login form text
    "sign in to your account", "create an account",
    "forgot your password", "reset your password",
    "注册登录即代表",
    # Specific login form components (multi-word to avoid false positives)
    "verify your phone", "verification code sent",
]

# Page content patterns that indicate SUCCESSFUL login (chat interface visible)
LOGGED_IN_INDICATORS = [
    "new chat", "new conversation",
    "message deepseek", "deepseek 给你",
    "send a message", "ask anything",
    "deep thinking", "深度思考",
    "what can i help with",
    "search the web", "联网搜索",
]


def extract_with_visible_browser(provider: str, timeout: int = 300) -> PersistedSession:

    url = PROVIDER_URLS.get(provider.lower())
    if not url:
        raise ValueError(f"Unknown provider: {provider}. Available: {list(PROVIDER_URLS.keys())}")

    print("=" * 60)
    print("🔐 Nexus Cookie Extractor")
    print("=" * 60)
    print(f"\n📍 目标网站: {url}")
    print("⏳ 等待手动登录...")
    print("\n操作步骤:")
    print("  1. 浏览器将自动打开目标网站")
    print("  2. 在浏览器中完成登录（支持邮箱/手机/验证码）")
    print("  3. 登录成功后脚本会自动提取 Cookie")
    print(f"  4. 超时时间: {timeout} 秒\n")
    print("=" * 60)

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=False)
        context = browser.new_context()
        page = context.new_page()

        print(f"🌐 正在打开 {url} ...")
        page.goto(url, timeout=timeout * 1000)

        # Wait for page to settle (redirects, JavaScript)
        print("⏳ 等待页面加载...")
        time.sleep(3)

        success_patterns = LOGIN_SUCCESS_PATTERNS.get(provider.lower(), [])
        failure_patterns = LOGIN_FAILURE_PATTERNS.get(provider.lower(), [])

        waited = 0
        print("\n🔎 监听登录状态...")
        last_url = ""
        while waited < timeout:
            current_url = page.url
            # Normalize URL: remove trailing slash for comparison
            normalized_url = current_url.rstrip("/")

            is_failure_page = any(p in normalized_url for p in failure_patterns)
            is_success_pattern = any(
                normalized_url.startswith(p.rstrip("/")) or normalized_url.rstrip("/").startswith(p.rstrip("/"))
                for p in success_patterns
            )

            if is_success_pattern and not is_failure_page:
                # Two-phase content verification
                try:
                    body_text = page.evaluate(
                        "() => (document.body?.innerText || '')"
                    )
                    body_lower = body_text.lower()

                    has_login_form = any(indicator.lower() in body_lower for indicator in LOGIN_FORM_INDICATORS)
                    has_logged_in = any(indicator.lower() in body_lower for indicator in LOGGED_IN_INDICATORS)

                    # Also check for chat textarea (strong signal)
                    has_chat_input = bool(page.evaluate(
                        "() => !!document.querySelector('textarea, [contenteditable=\"true\"], [role=\"textbox\"]')"
                    ))

                    if has_logged_in or (has_chat_input and not has_login_form):
                        logged_in_reason = "chat interface found" if has_chat_input else "logged-in content matched"
                        print(f"\n✅ 检测到登录成功! URL: {current_url}")
                        print(f"   判定依据: {logged_in_reason}")
                        print("📦 正在提取 Cookie...")
                        time.sleep(2)

                        cookies = context.cookies()
                        cookie_dict = {c["name"]: c["value"] for c in cookies}

                        essential_cookies = {
                            k: v for k, v in cookie_dict.items()
                            if len(v) > 5
                        }

                        browser.close()
                        return PersistedSession(cookies=essential_cookies, auth_tokens={})

                except Exception:
                    pass

            if current_url != last_url:
                if is_failure_page:
                    print(f"   🔒 当前在登录页: {current_url}")
                else:
                    print(f"   📍 URL 变化: {current_url} （非登录页，等待页面切换...）")
                last_url = current_url

            time.sleep(1)
            waited += 1

            if waited % 15 == 0:
                print(f"   ⏱ 已等待 {waited} 秒... 当前: {current_url}")
            if waited % 60 == 0:
                print(f"   💡 提示: 请在打开的浏览器窗口中完成登录")

        browser.close()
        raise TimeoutError(f"登录超时（{timeout}秒），请重试")


def parse_chrome_cookies(path: str) -> PersistedSession:
    with open(path, 'r') as f:
        data = json.load(f)
    cookies = {}
    if isinstance(data, list):
        for c in data:
            if isinstance(c, dict) and 'name' in c and 'value' in c:
                cookies[c['name']] = c['value']
    return PersistedSession(cookies=cookies, auth_tokens={})


def main():
    parser = argparse.ArgumentParser(description="Nexus Cookie Extractor")
    parser.add_argument("--provider", "-p", required=True,
                        choices=["claude", "chatgpt", "deepseek", "google"],
                        help="Provider (claude/chatgpt/deepseek/google)")
    parser.add_argument("--chrome-cookies", "-c", help="Import from Chrome cookies JSON")
    parser.add_argument("--output", "-o", help="Output file path")
    parser.add_argument("--timeout", "-t", type=int, default=300, help="Timeout in seconds")

    args = parser.parse_args()

    try:
        if args.chrome_cookies:
            session = parse_chrome_cookies(args.chrome_cookies)
        else:
            session = extract_with_visible_browser(args.provider, args.timeout)

        output = session.to_json()

        if args.output:
            with open(args.output, 'w') as f:
                f.write(output)
            print(f"\n💾 Cookie 已保存到: {args.output}")
        else:
            print("\n📋 Cookie JSON:")
            print(output)

        print(f"\n✅ 提取完成! 共 {len(session.cookies)} 个 Cookie")

    except Exception as e:
        print(f"\n❌ 错误: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()