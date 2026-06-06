Set-Content "C:\Users\ADMIN\vision-agent\src-tauri\src\agent.rs" @'
use reqwest::Client;
use serde_json::{json, Value};

pub async fn run_agent(
    prompt: &str,
    screenshot_b64: Option<&str>,
    installed_apps: &[String],
) -> Result<Vec<Value>, String> {
    let client = Client::new();

    // Security: cap app list to avoid context overflow
    let app_list = installed_apps
        .iter()
        .take(50)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    // Security: truncate prompt if somehow too long
    let safe_prompt = if prompt.len() > 500 {
        &prompt[..500]
    } else {
        prompt
    };

    let system_content = format!(
        r#"/no_think
You are a Windows automation agent. Output ONLY a JSON array of actions. No explanation.
You MUST NOT run any system commands, delete files, access passwords, or modify system settings.
You MUST NOT use cmd, powershell, registry, or any shell commands.
Only interact with apps through their normal UI.

INSTALLED APPS ON THIS MACHINE: {}

ACTIONS:
{{"tool":"open_app","args":{{"app":"<exact app name from list above>"}}}}
{{"tool":"screenshot","args":{{}}}}
{{"tool":"click","args":{{"x":100,"y":200}}}}
{{"tool":"type_text","args":{{"text":"hello"}}}}
{{"tool":"key_press","args":{{"keys":["enter"]}}}}
{{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}}
{{"tool":"wait","args":{{"ms":1000}}}}
{{"tool":"done","args":{{"message":"task complete"}}}}

APP SHORTCUTS (always use these instead of clicking):
- Spotify search: hotkey ctrl+l, then type_text, then enter
- Spotify play/pause: key_press space
- Browser address bar: hotkey ctrl+l
- New tab: hotkey ctrl+t

EXAMPLE - "open spotify and search Flashing Lights":
[
  {{"tool":"open_app","args":{{"app":"Spotify"}}}},
  {{"tool":"wait","args":{{"ms":4000}}}},
  {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}},
  {{"tool":"wait","args":{{"ms":500}}}},
  {{"tool":"type_text","args":{{"text":"Flashing Lights"}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"done","args":{{"message":"Opened Spotify and searched Flashing Lights"}}}}
]

EXAMPLE - "open notepad and type hello":
[
  {{"tool":"open_app","args":{{"app":"notepad"}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"type_text","args":{{"text":"hello"}}}},
  {{"tool":"done","args":{{"message":"Opened Notepad and typed hello"}}}}
]

ONLY output a JSON array. Nothing else."#,
        app_list
    );

    let user_content = match screenshot_b64 {
        None => json!([
            { "type": "text", "text": safe_prompt }
        ]),
        Some(b64) => json!([
            {
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", b64)
                }
            },
            {
                "type": "text",
                "text": format!("Current screen shown above. Original command: {}", safe_prompt)
            }
        ]),
    };

    let response = client
        .post("http://localhost:8081/v1/chat/completions")
        .timeout(std::time::Duration::from_secs(60))
        .json(&json!({
            "model": "qwen",
            "max_tokens": 2048,
            "temperature": 0.1,
            "top_p": 0.9,
            "messages": [
                { "role": "system", "content": system_content },
                { "role": "user",   "content": user_content }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    // Security: check response status
    if !response.status().is_success() {
        return Err(format!("LLM server error: {}", response.status()));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in LLM response")?;

    // Security: cap response size
    if text.len() > 10_000 {
        return Err("⛔ LLM response too large, possible attack".to_string());
    }

    // Strip thinking tags if present
    let text = if let Some(end) = text.find("</think>") {
        &text[end + 8..]
    } else {
        text
    };

    let clean = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(clean)
        .map_err(|e| format!("JSON parse error: {}\nRaw: {}", e, clean))
}
'@