use base64::{engine::general_purpose, Engine as _};
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use serde::Deserialize;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use xcap::Monitor;

#[derive(Debug, Deserialize)]
#[serde(tag = "tool", content = "args")]
pub enum ToolCall {
    #[serde(rename = "open_app")]
    OpenApp { app: String },

    #[serde(rename = "screenshot")]
    Screenshot,

    #[serde(rename = "click")]
    Click { x: i32, y: i32 },

    #[serde(rename = "right_click")]
    RightClick { x: i32, y: i32 },

    #[serde(rename = "double_click")]
    DoubleClick { x: i32, y: i32 },

    #[serde(rename = "type_text")]
    TypeText { text: String },

    #[serde(rename = "key_press")]
    KeyPress { keys: Vec<String> },

    #[serde(rename = "hotkey")]
    Hotkey { keys: Vec<String> },

    #[serde(rename = "wait")]
    Wait { ms: u64 },

    #[serde(rename = "scroll")]
    Scroll { x: i32, y: i32, direction: String },
}

pub fn execute_tool(call: &ToolCall) -> Result<String, String> {
    match call {
        ToolCall::OpenApp { app } => {
            Command::new("powershell")
                .args(["-Command", &format!("Start-Process '{}'", app)])
                .spawn()
                .map_err(|e| e.to_string())?;
            Ok(format!("🚀 Opened {}", app))
        }

        ToolCall::Screenshot => {
            let monitors = Monitor::all().map_err(|e| e.to_string())?;
            let monitor = monitors.into_iter().next().ok_or("No monitor found")?;
            let image = monitor.capture_image().map_err(|e| e.to_string())?;

            let mut buf = Vec::new();
            image
                .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;

            let b64 = general_purpose::STANDARD.encode(&buf);
            Ok(format!("SCREENSHOT:{}", b64))
        }

        ToolCall::Click { x, y } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo
                .move_mouse(*x, *y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            Ok(format!("🖱️ Clicked ({}, {})", x, y))
        }

        ToolCall::RightClick { x, y } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo
                .move_mouse(*x, *y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            enigo
                .button(Button::Right, Direction::Click)
                .map_err(|e| e.to_string())?;
            Ok(format!("🖱️ Right clicked ({}, {})", x, y))
        }

        ToolCall::DoubleClick { x, y } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo
                .move_mouse(*x, *y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            sleep(Duration::from_millis(80));
            enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            Ok(format!("🖱️ Double clicked ({}, {})", x, y))
        }

        ToolCall::TypeText { text } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo.text(text).map_err(|e| e.to_string())?;
            Ok(format!("⌨️ Typed: {}", text))
        }

        ToolCall::KeyPress { keys } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            for key in keys {
                let k = parse_key(key)?;
                enigo.key(k, Direction::Click).map_err(|e| e.to_string())?;
            }
            Ok(format!("⌨️ Pressed: {:?}", keys))
        }

        ToolCall::Hotkey { keys } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            let parsed: Result<Vec<Key>, String> = keys.iter().map(|k| parse_key(k)).collect();
            let parsed = parsed?;

            for k in &parsed[..parsed.len() - 1] {
                enigo.key(*k, Direction::Press).map_err(|e| e.to_string())?;
            }
            enigo
                .key(*parsed.last().unwrap(), Direction::Click)
                .map_err(|e| e.to_string())?;
            for k in parsed[..parsed.len() - 1].iter().rev() {
                enigo
                    .key(*k, Direction::Release)
                    .map_err(|e| e.to_string())?;
            }
            Ok(format!("⌨️ Hotkey: {:?}", keys))
        }

        ToolCall::Wait { ms } => {
            sleep(Duration::from_millis(*ms));
            Ok(format!("⏳ Waited {}ms", ms))
        }

        ToolCall::Scroll { x, y, direction } => {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            enigo
                .move_mouse(*x, *y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            let length = if direction == "down" { -3 } else { 3 };
            enigo
                .scroll(length, enigo::Axis::Vertical)
                .map_err(|e| e.to_string())?;
            Ok(format!("🖱️ Scrolled {} at ({}, {})", direction, x, y))
        }
    }
}

fn parse_key(key: &str) -> Result<Key, String> {
    match key.to_lowercase().as_str() {
        "enter" => Ok(Key::Return),
        "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "tab" => Ok(Key::Tab),
        "escape" => Ok(Key::Escape),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "win" => Ok(Key::Meta),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "delete" => Ok(Key::Delete),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        other => Err(format!("Unknown key: {}", other)),
    }
}
