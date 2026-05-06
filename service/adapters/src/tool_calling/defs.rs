use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParam>,
}

impl ToolDef {
    fn to_compact_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        for p in &self.parameters {
            properties.insert(
                p.name.clone(),
                serde_json::json!({
                    "type": p.param_type,
                    "description": p.description,
                }),
            );
            if p.required {
                required.push(p.name.clone());
            }
        }
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": required,
            }
        })
    }
}

pub fn default_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "web_search".into(),
            description: "Search the web for current information".into(),
            parameters: vec![ToolParam {
                name: "query".into(),
                param_type: "string".into(),
                description: "Search query string".into(),
                required: true,
            }],
        },
        ToolDef {
            name: "read".into(),
            description: "Read content from a file or URL".into(),
            parameters: vec![
                ToolParam {
                    name: "path".into(),
                    param_type: "string".into(),
                    description: "File path or URL to read".into(),
                    required: true,
                },
                ToolParam {
                    name: "offset".into(),
                    param_type: "integer".into(),
                    description: "Starting line number".into(),
                    required: false,
                },
                ToolParam {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    description: "Max lines to return".into(),
                    required: false,
                },
            ],
        },
        ToolDef {
            name: "write".into(),
            description: "Write content to a file".into(),
            parameters: vec![
                ToolParam {
                    name: "path".into(),
                    param_type: "string".into(),
                    description: "File path to write to".into(),
                    required: true,
                },
                ToolParam {
                    name: "content".into(),
                    param_type: "string".into(),
                    description: "Content to write".into(),
                    required: true,
                },
            ],
        },
        ToolDef {
            name: "exec".into(),
            description: "Execute a shell command".into(),
            parameters: vec![
                ToolParam {
                    name: "command".into(),
                    param_type: "string".into(),
                    description: "Shell command to execute".into(),
                    required: true,
                },
                ToolParam {
                    name: "workdir".into(),
                    param_type: "string".into(),
                    description: "Working directory for the command".into(),
                    required: false,
                },
            ],
        },
        ToolDef {
            name: "message".into(),
            description: "Send a message to the user (use for final responses)".into(),
            parameters: vec![ToolParam {
                name: "text".into(),
                param_type: "string".into(),
                description: "Message to send".into(),
                required: true,
            }],
        },
    ]
}

pub fn compact_tools_json(tools: &[ToolDef]) -> String {
    let schemas: Vec<_> = tools.iter().map(|t| t.to_compact_schema()).collect();
    serde_json::to_string(&schemas).unwrap_or_default()
}
