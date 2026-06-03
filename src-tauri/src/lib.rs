mod agent;
mod tools;

#[tauri::command]
async fn run_prompt(prompt: String) -> Result<Vec<String>, String> {
    let mut results: Vec<String> = vec![];
    let mut last_screenshot: Option<String> = None;
    let max_steps = 20;

    for _ in 0..max_steps {
        let tool_calls = agent::run_agent(&prompt, last_screenshot.as_deref()).await?;

        last_screenshot = None;

        for call_json in &tool_calls {
            let tool = call_json["tool"].as_str().unwrap_or("");

            if tool == "done" {
                let msg = call_json["args"]["message"]
                    .as_str()
                    .unwrap_or("Task complete")
                    .to_string();
                results.push(format!("✅ {}", msg));
                return Ok(results);
            }

            let call: tools::ToolCall = serde_json::from_value(call_json.clone())
                .map_err(|e| format!("Bad tool call: {}", e))?;

            let result = tools::execute_tool(&call)?;

            if result.starts_with("SCREENSHOT:") {
                last_screenshot = Some(result[11..].to_string());
                results.push("📸 Screenshot taken".to_string());
            } else {
                results.push(result);
            }
        }
    }

    Err("Agent reached max steps without completing".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_prompt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
