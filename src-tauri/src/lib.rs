mod agent;
mod tools;

use winreg;

const BLOCKED_KEYWORDS: &[&str] = &[
    "delete",
    "format",
    "rmdir",
    "rm -rf",
    "del ",
    "cmd",
    "powershell",
    "registry",
    "regedit",
    "shutdown",
    "restart",
    "taskkill",
    "net user",
    "netsh",
    "bcdedit",
    "diskpart",
    "cipher",
    "password",
    "credential",
    "shadow",
    "wmic",
    "attrib",
    "icacls",
    "takeown",
    "runas",
    "schtasks",
    "sc stop",
    "sc delete",
];

const ALLOWED_TOOLS: &[&str] = &[
    "open_app",
    "screenshot",
    "click",
    "double_click",
    "right_click",
    "type_text",
    "key_press",
    "hotkey",
    "wait",
    "scroll",
    "done",
];

fn is_safe_prompt(prompt: &str) -> Result<(), String> {
    let lower = prompt.to_lowercase();
    if prompt.trim().is_empty() {
        return Err("Prompt cannot be empty".to_string());
    }
    if prompt.len() > 500 {
        return Err("Prompt too long (max 500 characters)".to_string());
    }
    for keyword in BLOCKED_KEYWORDS {
        if lower.contains(keyword) {
            return Err(format!(
                "⛔ Blocked: prompt contains restricted keyword '{}'",
                keyword
            ));
        }
    }
    let injection_patterns = [
        "ignore previous",
        "ignore instructions",
        "system prompt",
        "jailbreak",
        "pretend you are",
        "act as",
        "you are now",
        "disregard",
    ];
    for pattern in &injection_patterns {
        if lower.contains(pattern) {
            return Err("⛔ Blocked: potential prompt injection detected".to_string());
        }
    }
    Ok(())
}

fn is_safe_tool(tool: &str) -> bool {
    ALLOWED_TOOLS.contains(&tool)
}

fn get_installed_apps() -> Vec<String> {
    let mut apps = Vec::new();
    let paths = [
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ];
    for path in &paths {
        if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE).open_subkey(path)
        {
            for subkey_name in key.enum_keys().flatten() {
                if let Ok(subkey) = key.open_subkey(&subkey_name) {
                    let name: Result<String, _> = subkey.get_value("DisplayName");
                    if let Ok(name) = name {
                        if !name.trim().is_empty() {
                            apps.push(name.trim().to_string());
                        }
                    }
                }
            }
        }
    }
    if let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
    {
        for subkey_name in key.enum_keys().flatten() {
            if let Ok(subkey) = key.open_subkey(&subkey_name) {
                let name: Result<String, _> = subkey.get_value("DisplayName");
                if let Ok(name) = name {
                    if !name.trim().is_empty() {
                        apps.push(name.trim().to_string());
                    }
                }
            }
        }
    }
    apps.sort();
    apps.dedup();
    apps
}

#[tauri::command]
async fn run_prompt(prompt: String) -> Result<Vec<String>, String> {
    is_safe_prompt(&prompt)?;

    let installed_apps = get_installed_apps();
    let mut results: Vec<String> = vec![];
    let mut last_screenshot: Option<String> = None;
    let max_steps = 15;

    for step_num in 0..max_steps {
        if step_num >= max_steps {
            return Err("⛔ Safety limit: max steps reached".to_string());
        }

        let tool_calls =
            agent::run_agent(&prompt, last_screenshot.as_deref(), &installed_apps).await?;

        if tool_calls.len() > 10 {
            return Err("⛔ Safety limit: too many tool calls in one response".to_string());
        }

        last_screenshot = None;

        for call_json in &tool_calls {
            let tool = call_json["tool"].as_str().unwrap_or("");

            if !is_safe_tool(tool) {
                return Err(format!("⛔ Blocked: unknown tool '{}'", tool));
            }

            if tool == "done" {
                let msg = call_json["args"]["message"]
                    .as_str()
                    .unwrap_or("Task complete")
                    .to_string();
                results.push(format!("✅ {}", msg));
                return Ok(results);
            }

            if tool == "type_text" {
                let text = call_json["args"]["text"].as_str().unwrap_or("");
                let lower = text.to_lowercase();
                for keyword in BLOCKED_KEYWORDS {
                    if lower.contains(keyword) {
                        return Err(format!(
                            "⛔ Blocked: agent tried to type restricted content '{}'",
                            keyword
                        ));
                    }
                }
            }

            if tool == "hotkey" || tool == "key_press" {
                if let Some(keys) = call_json["args"]["keys"].as_array() {
                    let key_str: Vec<&str> = keys.iter().filter_map(|k| k.as_str()).collect();
                    let combo = key_str.join("+").to_lowercase();
                    let blocked_combos =
                        ["win+r", "alt+f4", "ctrl+alt+delete", "ctrl+alt+t", "win+x"];
                    for blocked in &blocked_combos {
                        if combo.contains(blocked) {
                            return Err(format!(
                                "⛔ Blocked: restricted key combination '{}'",
                                combo
                            ));
                        }
                    }
                }
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
