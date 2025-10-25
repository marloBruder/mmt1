import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

class GlobalState {
  databaseState: DatabaseState | null = $state(null);
  databaseBeingOpened: string = $state("");
  lastEditorContent: string = $state("");
}

export class DatabaseState {
  databaseId: number = $state(0);
  databasePath: string = $state("");
  grammarCalculationsProgress: number = $state(0);
  grammarCalculationsError: boolean = $state(false);
  theoremAmount: number = $state(0);

  constructor(databaseId: number, databasePath: string, theoremAmount: number) {
    this.databaseId = databaseId;
    this.databasePath = databasePath;
    this.theoremAmount = theoremAmount;
  }
}

let globalState = new GlobalState();

listen("grammar-calculations-progress", (e) => {
  let [progress, databaseId] = e.payload as [number, number];
  if (globalState.databaseState !== null && globalState.databaseState.databaseId === databaseId && progress > globalState.databaseState.grammarCalculationsProgress) {
    globalState.databaseState.grammarCalculationsProgress = progress;
  }
});

async function loadExternalWindowRelevantInfo() {
  let theoremAmount = (await invoke("load_external_window_relevant_info")) as number;
  globalState.databaseState = new DatabaseState(0, "", theoremAmount);
}

export { globalState, loadExternalWindowRelevantInfo };
