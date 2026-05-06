use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: Value,
}

#[derive(Debug)]
#[allow(dead_code)]
enum ParseState {
    Normal,
    InFence { fence_type: String, buffer: String },
}

pub fn extract_tool_calls(text: &str) -> Vec<ToolCall> {
    let mut calls = Vec::new();

    if let Some(call) = try_parse_fenced_json(text, "tool_json") {
        calls.push(call);
    }

    if let Some(call) = try_parse_fenced_json(text, "tool_call") {
        calls.push(call);
    }

    if let Some(call) = try_parse_fenced_json(text, "json") {
        if calls.is_empty() {
            calls.push(call);
        }
    }

    if let Some(call) = try_parse_xml_tool_call(text) {
        if calls.is_empty() {
            calls.push(call);
        }
    }

    calls
}

fn try_parse_fenced_json(text: &str, fence_label: &str) -> Option<ToolCall> {
    let start_marker = format!("```{}", fence_label);
    let start = text.find(&start_marker)?;

    let json_start = start + start_marker.len();
    let remaining = &text[json_start..];

    let end = remaining.find("```")?;
    let json_str = &remaining[..end].trim();

    if json_str.is_empty() {
        return None;
    }

    parse_tool_json(json_str).ok()
}

fn try_parse_xml_tool_call(text: &str) -> Option<ToolCall> {
    let start_tag = "<tool_call>";
    let end_tag = "</tool_call>";

    let start = text.find(start_tag)?;
    let json_start = start + start_tag.len();
    let remaining = &text[json_start..];
    let end = remaining.find(end_tag)?;
    let inner = &remaining[..end].trim();

    if inner.is_empty() {
        return None;
    }

    parse_tool_json(inner).ok()
}

fn parse_tool_json(json_str: &str) -> Result<ToolCall, String> {
    let mut repaired = json_str.to_string();
    if !repaired.trim().ends_with('}') {
        let mut depth = 0;
        for c in repaired.chars() {
            if c == '{' { depth += 1; }
            if c == '}' { depth -= 1; }
        }
        for _ in 0..depth {
            repaired.push('}');
        }
    }

    let v: Value = serde_json::from_str(&repaired)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let tool = v.get("tool")
        .or_else(|| v.get("name"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Missing 'tool' or 'name' field".to_string())?;

    let args = v.get("args")
        .or_else(|| v.get("arguments"))
        .or_else(|| v.get("parameters"))
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    Ok(ToolCall { tool, args })
}

pub fn has_tool_call(text: &str) -> bool {
    text.contains("```tool_json")
        || text.contains("```tool_call")
        || text.contains("<tool_call>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fenced_json() {
        let text = r#"Let me search for that.
```tool_json
{"tool": "web_search", "args": {"query": "weather Tokyo"}}
```"#;
        let calls = extract_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool, "web_search");
    }

    #[test]
    fn test_parse_xml() {
        let text = r#"<tool_call>
{"tool": "read", "args": {"path": "/tmp/test"}}
</tool_call>"#;
        let calls = extract_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool, "read");
    }

    #[test]
    fn test_no_tool_call() {
        let text = "The answer is 42.";
        let calls = extract_tool_calls(text);
        assert!(calls.is_empty());
    }
}
