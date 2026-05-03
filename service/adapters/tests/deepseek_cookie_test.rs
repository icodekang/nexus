//! DeepSeek Cookie Verification Test
//!
//! Run with: cargo test --test deepseek_cookie_test -- --nocapture
//! (Requires Chrome/Chromium installed on the system)

use std::collections::HashMap;
use std::time::Duration;

/// The cookie JSON from the user
const COOKIE_JSON: &str = r#"{"cookies":{"HWWAFSESTIME":"1777804867487","HWWAFSESID":"9b260bc5d9db13416bf","ds_session_id":"cd6e848ff29d4a8dbf6a7499661c0f39","smidV2":"2026050318411067501ab73542f63b6cef051928127aec00ca211a4a95b6050",".thumbcache_6b2e5483f9d858d7c661c5e276b6a6ae":"ZFVTZHkgnohcUOm8pX0t8p3cu8T1mJSvYUpyrNi4aTlOqjYEpCFEypO3ag/6Z1xeTKkB+d2cVLSUlLThdygCOQ%3D%3D"},"auth_tokens":{},"expires_at":null}"#;

#[derive(serde::Deserialize)]
struct CookieData {
    cookies: HashMap<String, String>,
    #[allow(dead_code)]
    auth_tokens: HashMap<String, String>,
    #[allow(dead_code)]
    expires_at: Option<String>,
}

#[tokio::test]
#[ignore = "requires headless Chrome and valid DeepSeek session"]
async fn verify_deepseek_cookies_can_chat() {
    // Parse the cookie JSON
    let data: CookieData = serde_json::from_str(COOKIE_JSON)
        .expect("Failed to parse cookie JSON");

    println!("\n=== DeepSeek Cookie Verification ===\n");
    println!("Cookies loaded: {}", data.cookies.len());
    for (k, v) in &data.cookies {
        let truncated = if v.len() > 50 {
            format!("{}...", &v[..50])
        } else {
            v.clone()
        };
        println!("  {} = {}", k, truncated);
    }

    // Check for essential session cookie
    if !data.cookies.contains_key("ds_session_id") {
        panic!("FAIL: 'ds_session_id' cookie is missing! Required for authentication.");
    }

    // Launch headless Chrome
    println!("\nLaunching headless Chrome...");
    let browser = match headless_chrome::Browser::default() {
        Ok(b) => b,
        Err(e) => {
            println!("SKIP: Cannot launch Chrome: {}. Check CHROME_PATH or install Chrome.", e);
            return;
        }
    };

    println!("Opening tab and navigating to DeepSeek...");
    let tab = browser.new_tab().expect("Failed to create tab");

    // Navigate to DeepSeek home page first (to set domain)
    tab.navigate_to("https://chat.deepseek.com/")
        .expect("Failed to navigate to DeepSeek");
    println!("Navigated to https://chat.deepseek.com/");
    std::thread::sleep(Duration::from_secs(3));

    // Inject cookies
    println!("\nInjecting {} cookies...", data.cookies.len());
    let cookie_pairs_json = serde_json::to_string(
        &data.cookies.iter()
            .map(|(k, v)| [k.as_str(), v.as_str()])
            .collect::<Vec<_>>()
    ).unwrap();

    let cookies_script = format!(
        r#"
        (function() {{
            const pairs = {};
            pairs.forEach(function(pair) {{
                document.cookie = pair[0] + '=' + pair[1] + '; path=/; domain=.deepseek.com';
            }});
            return document.cookie;
        }})();
        "#,
        cookie_pairs_json
    );

    match tab.evaluate(&cookies_script, false) {
        Ok(result) => {
            let cookies_set = result.value
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            println!("Cookies after injection: {} chars", cookies_set.len());
        }
        Err(e) => {
            panic!("FAIL: Cookie injection JS failed: {}", e);
        }
    }

    // RELOAD the page so cookies are sent to server for authentication
    println!("\nReloading page with cookies to activate session...");
    tab.navigate_to("https://chat.deepseek.com/")
        .expect("Failed to reload DeepSeek");
    std::thread::sleep(Duration::from_secs(5));

    // Check page state
    let check_script = r#"
        (function() {
            return JSON.stringify({
                url: window.location.href,
                title: document.title,
                bodyPreview: (document.body?.innerText || '').substring(0, 300)
            });
        })();
    "#;

    match tab.evaluate(check_script, false) {
        Ok(result) => {
            if let Some(serde_json::Value::String(s)) = &result.value {
                println!("Page after reload: {}", s);
                let lower = s.to_lowercase();
                if lower.contains("log in") || lower.contains("sign in") || lower.contains("sign_up") || lower.contains("登录") {
                    panic!("FAIL: Still on login/register page. Cookies may be expired or invalid.\nPage info: {}", s);
                }
            }
        }
        Err(e) => {
            panic!("FAIL: Cannot check page state: {}", e);
        }
    }

    println!("\nPage appears to be authenticated! Sending test message...");

    // Send a test chat message
    let test_script = r#"
        (async () => {
            const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                window.HTMLTextAreaElement.prototype, 'value'
            ).set;

            // Find textarea
            const inputEl = document.querySelector('textarea');
            if (!inputEl) {
                return JSON.stringify({ error: "no textarea found" });
            }

            // Type the message using React-compatible setter
            const testMsg = "Hi, reply with just the word OK please.";
            inputEl.focus();
            nativeInputValueSetter.call(inputEl, testMsg);
            inputEl.dispatchEvent(new Event('input', { bubbles: true }));
            inputEl.dispatchEvent(new Event('change', { bubbles: true }));

            await new Promise(r => setTimeout(r, 1000));

            // Click send button
            const buttons = Array.from(document.querySelectorAll('button'));
            const sendBtn = buttons.find(b => {
                const text = b.textContent.trim().toLowerCase();
                return text === 'send' || text === '发送' || b.querySelector('svg');
            });

            if (sendBtn) {
                sendBtn.click();
            } else {
                inputEl.dispatchEvent(new KeyboardEvent('keydown', {
                    key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true
                }));
            }

            // Wait for response
            await new Promise(r => setTimeout(r, 20000));

            // Get response
            const selectors = [
                '.ds-markdown',
                '[data-node-id]',
                '[class*="message"] [class*="content"]'
            ];

            let responseText = '';
            for (const sel of selectors) {
                const els = document.querySelectorAll(sel);
                if (els.length > 0) {
                    const last = els[els.length - 1];
                    responseText = last.textContent.trim();
                    if (responseText.length > 10) break;
                }
            }

            if (!responseText) {
                const allDivs = document.querySelectorAll('[class*="ds-"]');
                if (allDivs.length > 0) {
                    responseText = allDivs[allDivs.length - 1].textContent.trim().substring(0, 500);
                }
            }

            return JSON.stringify({
                hasResponse: responseText.length > 0,
                responsePreview: responseText.substring(0, 300),
                responseLength: responseText.length
            });
        })();
    "#;

    match tab.evaluate(test_script, false) {
        Ok(result) => {
            let response_text = result.value
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| r#"{"error":"No return value"}"#.to_string());

            println!("\nChat response: {}", response_text);

            let parsed: serde_json::Value = serde_json::from_str(&response_text)
                .unwrap_or_else(|_| serde_json::json!({"error": "parse failed", "raw": response_text}));

            if let Some(err) = parsed.get("error") {
                panic!("FAIL: Chat JS returned error: {}", err);
            }

            let has_response = parsed.get("hasResponse").and_then(|v| v.as_bool()).unwrap_or(false);
            let preview = parsed.get("responsePreview").and_then(|v| v.as_str()).unwrap_or("");
            let length = parsed.get("responseLength").and_then(|v| v.as_u64()).unwrap_or(0);

            if has_response && length > 5 {
                println!("\n✓ SUCCESS: DeepSeek chat responded!");
                println!("  Response length: {} chars", length);
                println!("  Preview: {}", preview);
            } else {
                panic!(
                    "FAIL: No meaningful response. has_response={}, length={}, preview={}",
                    has_response, length, preview
                );
            }
        }
        Err(e) => {
            panic!("FAIL: Chat JS execution failed: {}", e);
        }
    }

    println!("\n=== Verification Complete ===\n");
}
