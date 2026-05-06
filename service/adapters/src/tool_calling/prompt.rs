use super::defs::{compact_tools_json, ToolDef};

const TOOL_CALLING_SYSTEM: &str = r#"You have access to the following tools. Use them by outputting a JSON block wrapped in ```tool_json code fences.

Available tools:
{tools_schema}

Example:
```tool_json
{"tool": "web_search", "args": {"query": "current weather in Tokyo"}}
```

Rules:
- Only call one tool per response
- The tool_json block must be on its own, with no other text
- After receiving tool output, you may call another tool or send a final message
- Always respond; never leave the user hanging
"#;

pub fn build_tool_system_prompt(tools: &[ToolDef], provider: &str) -> String {
    let tools_json = compact_tools_json(tools);
    match provider {
        "claude" | "claude.ai" => {
            format!(
                "You have tools available:\n{}\n\nTo use a tool, output ONLY:\n```tool_json\n{{\"tool\": \"<name>\", \"args\": {{...}}}}\n```\n\nWait for the tool result before continuing.",
                tools_json
            )
        }
        "chatgpt" | "chat.openai.com" | "openai" => {
            format!(
                "Tools:\n{}\n\nCall a tool by writing:\n```tool_json\n{{\"tool\": \"<name>\", \"args\": {{...}}}}\n```",
                tools_json
            )
        }
        "deepseek" => {
            format!(
                "可用工具：\n{}\n\n调用格式：\n```tool_json\n{{\"tool\": \"<名称>\", \"args\": {{...}}}}\n```\n\n每次只调用一个工具。",
                tools_json
            )
        }
        _ => TOOL_CALLING_SYSTEM.replace("{tools_schema}", &tools_json),
    }
}

pub fn inject_tool_prompt(
    messages: &[crate::types::Message],
    tools: &[ToolDef],
    provider: &str,
) -> Vec<crate::types::Message> {
    let system_prompt = build_tool_system_prompt(tools, provider);

    let mut new_messages = Vec::with_capacity(messages.len() + 1);
    new_messages.push(crate::types::Message::system(system_prompt));
    new_messages.extend_from_slice(messages);
    new_messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool_calling::defs::ToolParam;

    #[test]
    fn test_build_prompt() {
        let tools = vec![ToolDef {
            name: "web_search".into(),
            description: "Search web".into(),
            parameters: vec![ToolParam {
                name: "query".into(),
                param_type: "string".into(),
                description: "Query".into(),
                required: true,
            }],
        }];
        let prompt = build_tool_system_prompt(&tools, "claude");
        assert!(prompt.contains("web_search"));
        assert!(prompt.contains("```tool_json"));
    }
}
