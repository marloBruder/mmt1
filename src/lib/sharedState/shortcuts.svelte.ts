import { tabManager } from "./tabManager.svelte";

export function setupShortcuts() {
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key == "r") {
      e.preventDefault();
    } else if (e.ctrlKey && e.key == "p") {
      e.preventDefault();
    } else if (e.ctrlKey && e.key == "w") {
      tabManager.closeOpenTab();
    }
  });
}
