use reqwest::Client;
use serde_json::{json, Value};

pub async fn run_agent(
    prompt: &str,
    screenshot_b64: Option<&str>,
    installed_apps: &[String],
) -> Result<Vec<Value>, String> {
    let client = Client::new();

    let app_list = installed_apps
        .iter()
        .take(50)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    let safe_prompt = if prompt.len() > 500 {
        &prompt[..500]
    } else {
        prompt
    };

    let system_content = format!(
        r#"/no_think
YOU ARE A WINDOWS AUTOMATION AGENT.
YOUR ONLY JOB: Read the user command, output a JSON array of actions. NOTHING ELSE.

STRICT RULES:
1. OUTPUT ONLY a valid JSON array. NO text before or after. NO explanation. NO markdown.
2. ALWAYS end every response with the done tool.
3. ALWAYS use wait after open_app to let the app load.
4. ALWAYS use keyboard shortcuts instead of clicking when available.
5. NEVER guess coordinates. ONLY click if you have taken a screenshot first.
6. If the app is already open, do NOT open it again.
7. Use EXACT app names from the installed apps list below.

INSTALLED APPS: {}

AVAILABLE TOOLS:
open_app    -> {{"tool":"open_app","args":{{"app":"Spotify"}}}}
wait        -> {{"tool":"wait","args":{{"ms":2000}}}}
hotkey      -> {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}}
key_press   -> {{"tool":"key_press","args":{{"keys":["enter"]}}}}
type_text   -> {{"tool":"type_text","args":{{"text":"hello world"}}}}
screenshot  -> {{"tool":"screenshot","args":{{}}}}
click       -> {{"tool":"click","args":{{"x":100,"y":200}}}}
scroll      -> {{"tool":"scroll","args":{{"x":100,"y":200,"direction":"down"}}}}
done        -> {{"tool":"done","args":{{"message":"what you did"}}}}

APP SHORTCUTS:
SPOTIFY: search=ctrl+l | play/pause=space | next=ctrl+right | prev=ctrl+left | vol up=ctrl+up | vol down=ctrl+down
CHROME/EDGE: address bar=ctrl+l | new tab=ctrl+t | close tab=ctrl+w | reload=ctrl+r | find=ctrl+f
YOUTUBE: play/pause=k | fullscreen=f | mute=m | forward 10s=l | back 10s=j
VSCODE: terminal=ctrl+` | save=ctrl+s | find=ctrl+f | command palette=ctrl+shift+p | open file=ctrl+p
NOTEPAD: save=ctrl+s | select all=ctrl+a | find=ctrl+f | undo=ctrl+z
FILE EXPLORER: address bar=ctrl+l | new folder=ctrl+shift+n | rename=f2 | search=ctrl+f
DISCORD: search=ctrl+f | settings=ctrl+comma | upload=ctrl+u
WINDOWS: switch apps=alt+tab | show desktop=win+d | settings=win+i | screenshot=win+shift+s

EXAMPLES:

INPUT: "open spotify"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"Spotify"}}}},
  {{"tool":"wait","args":{{"ms":3000}}}},
  {{"tool":"done","args":{{"message":"Opened Spotify"}}}}
]

INPUT: "open spotify and search Flashing Lights"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"Spotify"}}}},
  {{"tool":"wait","args":{{"ms":4000}}}},
  {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}},
  {{"tool":"wait","args":{{"ms":500}}}},
  {{"tool":"type_text","args":{{"text":"Flashing Lights"}}}},
  {{"tool":"wait","args":{{"ms":1500}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"done","args":{{"message":"Opened Spotify and searched Flashing Lights"}}}}
]

INPUT: "play next song on spotify"
OUTPUT:
[
  {{"tool":"hotkey","args":{{"keys":["ctrl","right"]}}}},
  {{"tool":"done","args":{{"message":"Skipped to next song"}}}}
]

INPUT: "open chrome and go to youtube"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"Google Chrome"}}}},
  {{"tool":"wait","args":{{"ms":3000}}}},
  {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}},
  {{"tool":"wait","args":{{"ms":500}}}},
  {{"tool":"type_text","args":{{"text":"youtube.com"}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"done","args":{{"message":"Opened Chrome and navigated to YouTube"}}}}
]

INPUT: "open chrome and search cats on youtube"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"Google Chrome"}}}},
  {{"tool":"wait","args":{{"ms":3000}}}},
  {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}},
  {{"tool":"wait","args":{{"ms":500}}}},
  {{"tool":"type_text","args":{{"text":"youtube.com/results?search_query=cats"}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"done","args":{{"message":"Searched cats on YouTube"}}}}
]

INPUT: "open notepad and write hello world"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"notepad"}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"type_text","args":{{"text":"hello world"}}}},
  {{"tool":"done","args":{{"message":"Opened Notepad and typed hello world"}}}}
]

INPUT: "open file explorer"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"explorer"}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"done","args":{{"message":"Opened File Explorer"}}}}
]

INPUT: "search google for weather today"
OUTPUT:
[
  {{"tool":"open_app","args":{{"app":"Google Chrome"}}}},
  {{"tool":"wait","args":{{"ms":3000}}}},
  {{"tool":"hotkey","args":{{"keys":["ctrl","l"]}}}},
  {{"tool":"wait","args":{{"ms":500}}}},
  {{"tool":"type_text","args":{{"text":"google.com/search?q=weather today"}}}},
  {{"tool":"key_press","args":{{"keys":["enter"]}}}},
  {{"tool":"wait","args":{{"ms":2000}}}},
  {{"tool":"done","args":{{"message":"Searched Google for weather today"}}}}
]

REMEMBER: OUTPUT ONLY THE JSON ARRAY. NO OTHER TEXT."#,
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

    if text.len() > 10_000 {
        return Err("⛔ LLM response too large".to_string());
    }

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

    serde_json::from_str(clean).map_err(|e| format!("JSON parse error: {}\nRaw: {}", e, clean))
}
