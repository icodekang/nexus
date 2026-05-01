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
    "claude": ["/api/auth/session", "claude.ai", "/settings"],
    "chatgpt": ["chat.openai.com", "/settings", "/chat"],
    "deepseek": ["chat.deepseek.com"],
    "google": ["ai.google.dev", "/studio", "/gallery"],
}


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

        patterns = LOGIN_SUCCESS_PATTERNS.get(provider.lower(), ["/settings", "/chat"])

        waited = 0
        print("\n🔎 监听登录状态...")
        while waited < timeout:
            current_url = page.url

            for pattern in patterns:
                if pattern in current_url:
                    print(f"\n✅ 检测到登录成功! URL: {current_url}")
                    print("📦 正在提取 Cookie...")
                    time.sleep(1)

                    cookies = context.cookies()
                    cookie_dict = {c['name']: c['value'] for c in cookies}

                    browser.close()
                    return PersistedSession(cookies=cookie_dict, auth_tokens={})

            time.sleep(1)
            waited += 1

            if waited % 10 == 0:
                print(f"  已等待 {waited} 秒...")

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