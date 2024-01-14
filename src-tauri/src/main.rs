// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Serialize, Deserialize};
use tauri::Window;
use winapi::um::winuser::{GetAsyncKeyState, VK_F11, VK_F10};

/// A struct containing game information.
#[derive(Serialize, Deserialize)]
struct GameInfo {
    /// Actually looks like this: [title] by [author]
    title: String,
    /// URL where the game can be found.
    url: String,
    /// width of the game in pixels.
    width: u32,
    /// height of the game in pixels.
    height: u32,
}

/// find the first instance within a given string that is surrounded by
/// `opening` and `closing`.
/// 
/// Returns `None` if there is no such instance.
fn find_enclosed(html: &str, opening: &str, closing: &str) -> Option<String> {
    let mut split = html.split(opening);
    if let Some(frag) = split.nth(1) {
        match frag.split(closing).next() {
            Some(f) => Some(f.to_string()),
            None => None,
        }
    } else {
        None
    }
}

/// small wrapper function.
fn find_game(html: &str, domain: &str) -> Option<String> {
    let full_path = format!("src=\"{}", domain);
    if let Some(url) = find_enclosed(html, &full_path, "\"") {
        Some(format!("{}{}", domain, url))
    } else {
        None
    }
}

/// Given a `html` file, this function tries to extract game information.
/// 
/// Should hopefully only ever find valid game information for itch.io web
/// games. Returns `None` if the required information could not be found.
fn extract_game_info(html: &str) -> Option<GameInfo> {
    // two different domains are possible depending on the game.
    let url = if let Some(url) = find_game(&html, "https://html.itch.zone/html/") {
        url
    } else if let Some(url) = find_game(&html, "https://html-classic.itch.zone/html/") {
        url
    } else {
        return None;
    };
    // find and parse the other parameters.
    let title = find_enclosed(html, "<title>", "</title>");
    let title = match title {
        Some(t) => t,
        None => return None,
    };
    let width = find_enclosed(html, "data-width=\"", "\"");
    let width = match width {
        Some(w) => w,
        None => return None,
    };
    let width: u32 = match width.parse() {
        Ok(w) => w,
        Err(_) => return None,
    };
    let height = find_enclosed(html, "data-height=\"", "\"");
    let height = match height {
        Some(h) => h,
        None => return None,
    };
    let height: u32 = match height.parse() {
        Ok(h) => h,
        Err(_) => return None,
    };
    Some(GameInfo {
        title,
        url,
        width,
        height,
    })
}

/// Try to extract game information from the given `url`.
/// 
/// If any step fails, this function returns a more or less helpful error
/// message.
#[tauri::command]
async fn load_game(url: String) -> Result<GameInfo, String> {
    let resp = reqwest::get(url).await;
    if resp.is_err() {
        return Err("Could not complete HTTP request.".to_string());
    }
    let resp = resp.unwrap().text().await;
    if resp.is_err() {
        return Err("Invalid HTTP response.".to_string());
    }
    let html = resp.unwrap();
    let html = html.replace("&quot;", "\"");
    if let Some(info) = extract_game_info(&html) {
        Ok(info)
    } else {
        Err("Could not parse site.".to_string())
    }
}

/// Make the calling `window` listen to global events.
/// 
/// This function fires a `"home"` event when F10 is pressed and a
/// `"fullscreen"` event when F11 is pressed. Both events have empty payloads.
#[tauri::command]
fn listen_to_keyboard(window: Window) {
    std::thread::spawn(move || {
        let mut fullscreen_held = false;
        let mut home_held = false;
        loop {
            // ugly workaround that only works for windows, but unfortunately,
            // using mki didn't work because the webview seemt to trap keyboard
            // events.
            unsafe {
                let state = GetAsyncKeyState(VK_F11);
                if state != 0 && !fullscreen_held {
                    let _ = window.set_fullscreen(!window.is_fullscreen().unwrap_or(true));
                }
                fullscreen_held = state != 0;
                let state = GetAsyncKeyState(VK_F10);
                if state != 0 && !home_held {
                    let _ = window.eval("document.location = \"http://localhost:1420/\";");
                }
                home_held = state != 0;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });
}

/// Main function.
fn main() {
    println!("Hello!");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_game, listen_to_keyboard])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
