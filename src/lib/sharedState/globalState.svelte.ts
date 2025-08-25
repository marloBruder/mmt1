import { listen } from "@tauri-apps/api/event";

class GlobalState {
  databaseState: DatabaseState = new DatabaseState();
  databaseBeingOpened: string = $state("");
}

class DatabaseState {
  databaseId: number | null = $state(null);
  grammarCalculationsProgress: number = $state(0);
  theoremAmount: number = $state(0);
}

let globalState = new GlobalState();

listen("grammar-calculations-progress", (e) => {
  let [progress, databaseId] = e.payload as [number, number];
  if (globalState.databaseState.databaseId === databaseId && progress > globalState.databaseState.grammarCalculationsProgress) {
    globalState.databaseState.grammarCalculationsProgress = progress;
  }
});

export { globalState };
