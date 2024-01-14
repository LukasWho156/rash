//@ts-ignore
const TAURI: any = window.parent.__TAURI__;

// how many recent games should be shown at maximum
const MAX_GAMES: number = 8;

type GameInfo = {
  title: string,
  url: string,
  width: number,
  height: number
}

const btn = document.querySelector('#loadGameButton') as HTMLButtonElement;
const urlInput = document.querySelector('#loadGameUrl') as HTMLInputElement;
const form = document.querySelector('#loadGameForm') as HTMLFormElement;
const errorSpan = document.querySelector('#loadGameError') as HTMLSpanElement;
const recentGamesDiv = document.querySelector('#recentGames') as HTMLDivElement;

const recentGames: GameInfo[] = JSON.parse(localStorage.getItem("itchyDesktop.recent") ?? "[]");

// update the DOM to reflect recent games.
const setRecentGames = () => {
  recentGamesDiv.innerHTML = "&nbsp;";
  recentGames.forEach((game, i) => {
    if(i > 0) {
      const separator = document.createElement("span");
      separator.textContent = " - ";
      recentGamesDiv.appendChild(separator);
    }
    const link = document.createElement("a");
    link.textContent = game.title;
    link.href = game.url;
    link.addEventListener("click", () => adjustWindow(game));
    recentGamesDiv.appendChild(link);
  })
}

// check if a game is within the recent list and if not, add it.
const addGameToRecent = (game: GameInfo) => {
  if(recentGames.find(g => g.url === game.url)) return;
  recentGames.unshift(game);
  // truncate the recent list if it gets too long
  if(recentGames.length > MAX_GAMES) {
    recentGames.pop();
  }
  localStorage.setItem("itchyDesktop.recent", JSON.stringify(recentGames));
  setRecentGames();
}

// set window title and size when loading a new game.
const adjustWindow = async (game: GameInfo) => {
  TAURI.window.appWindow.setTitle(game.title);
  if(await TAURI.window.appWindow.isMaximized() || await TAURI.window.appWindow.isFullscreen()) {
    return;
  }
  const monitor = await TAURI.window.currentMonitor();
  const logical = monitor.size.toLogical(monitor.scaleFactor);
  if(logical.width < game.width || logical.height < game.height) {
    TAURI.window.appWindow.maximize();
    return;
  }
  TAURI.window.appWindow.setSize(new TAURI.window.LogicalSize(game.width, game.height)).then(() => {
    TAURI.window.appWindow.center();
  });
}

// try to load the game at the given URL
const loadGame = () => {
  const url = urlInput.value;
  btn.disabled = true;
  urlInput.disabled = true;
  errorSpan.innerHTML = "&nbsp;";
  TAURI.invoke("load_game", { url: url }).then((res: GameInfo) => {
    addGameToRecent(res);
    document.location = res.url;
    adjustWindow(res);
  }).catch((err: string) => {
    errorSpan.textContent = err;
  }).finally(() => {
    btn.disabled = false;
    urlInput.disabled = false;
  })
}

btn.addEventListener('click', loadGame);
form.addEventListener('submit', (e) => e.preventDefault());
setRecentGames();