import { invoke, window } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

// since iframes "trap" keyboard events, we'll have to use global keyboard events
// for general functionality (fullscreen, return to homepage);
invoke("listen_to_keyboard");

const toggleFullscreen = async () => {
    if(await window.appWindow.isFullscreen()) {
        window.appWindow.setFullscreen(false);
    } else {
        window.appWindow.setFullscreen(true);
    }
}

const returnToHomePage = async () => {
    (document.querySelector("#itchy-container") as HTMLIFrameElement).src = "src/start.html";
    window.appWindow.setTitle("Rash");
    window.currentMonitor()
}

listen("fullscreen", toggleFullscreen);
listen("home", returnToHomePage);