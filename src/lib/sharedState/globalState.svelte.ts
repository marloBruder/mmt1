import { listen } from "@tauri-apps/api/event";

class GlobalState {
  databaseBeingOpened: string = $state("");
  grammarCalculationsProgress: number = $state(0);
}

let globalState = new GlobalState();

listen("grammar-calculations-progress", (e) => {
  let [progress, databasePath] = e.payload as [number, string];
  if (globalState.databaseBeingOpened !== databasePath && progress > globalState.grammarCalculationsProgress) {
    globalState.grammarCalculationsProgress = progress;
  }
});

export { globalState };
