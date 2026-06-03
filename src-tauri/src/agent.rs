use reqwest::Client;
use serde_json::{json, Value};

const SYSTEM_PROMPT: &str = r#"
You are a Windows desktop automation agent with vision.
You receive a user command and optionally a screenshot of the current screen.

Respond ONLY with a valid JSON array of tool calls. No explanation, no markdown, no backticks.

Available tools:
- open_app:     { "tool": "open_app",     "args": { "app": "<name>" } }
- screenshot:   { "tool": "screenshot",   "args": {} }
- click:        { "tool": "click",        "args": { "x": 100, "y": 200 } }
- double_click: { "tool": "double_click", "args": { "x": 100, "y": 200 } }
- right_click:  { "tool": "right_click",  "args": { "x": 100, "y": 200 } }
- type_text:    { "tool": "type_text",    "args": { "text": "<string>" } }
- key_press:    { "tool": "key_press",    "args": { "keys": ["enter"] } }
- hotkey:       { "tool": "hotkey",       "args": { "keys": ["ctrl", "l"] } }
- scroll:       { "tool": "scroll",       "args": { "x": 100, "y": 200, "direction": "down" } }
- wait:         { "tool": "wait",         "args": { "ms": 1500 } }
- done:         { "tool": "done",         "args": { "message": "<what you did>" } }

Rules:
1. Always take a screenshot after opening an app before doing anything else.
2. Look at the screenshot carefully to find exact pixel coordinates of UI elements.
3. Always end with a done tool call.
4. If an action fails or the screen looks wrong, take another screenshot and reassess.
"#;

pub async fn run_agent(
    prompt: &str,
    screenshot_b64: Option<&str>,
) -> Result<Vec<Value>, String> {
    let client = Client::new();

    let user_content = match screenshot_b64 {
        None => json!([
            { "type": "text", "text": prompt }
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
                "text": format!("Current screen shown above. Original command: {}", prompt)
            }
        ]),
    };

    let response = client
        .post("http://localhost:8080/v1/chat/completions")
        .json(&json!({
            "model": "phi-4-mini",
            "max_tokens": 1024,
            "temperature": 0.1,
            "messages": [
                { "role": "system", "content": SYSTEM_PROMPT },
                { "role": "user",   "content": user_content }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    let body: Value = response.json().await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No content in LLM response")?;

    let clean = text.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(clean)
        .map_err(|e| format!("JSON parse error: {}\nRaw: {}", e, clean))
}