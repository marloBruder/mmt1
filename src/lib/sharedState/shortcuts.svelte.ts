import { confirm } from "@tauri-apps/plugin-dialog";
import { tabManager } from "./tabManager.svelte";

export function setupShortcuts() {
  document.addEventListener("keydown", async (e: KeyboardEvent) => {
    if (e.repeat) {
      return;
    }

    if (e.ctrlKey && e.key == "r") {
      e.preventDefault();
    } else if (e.ctrlKey && e.key == "p") {
      e.preventDefault();
    } else if (e.ctrlKey && e.key == "w") {
      let openTab = tabManager.getOpenTab();
      if (openTab !== null && openTab.showUnsavedChanges()) {
        if (!(await confirm("There are unsaved changes in this tab. Are you sure you want to close it?", { okLabel: "Close Tab", kind: "warning" }))) {
          return;
        }
      }

      tabManager.closeOpenTab();
    }
  });

  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
  });
}
