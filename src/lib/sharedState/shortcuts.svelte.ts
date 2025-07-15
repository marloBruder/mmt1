export function setupShortcuts() {
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.key == "r") {
      e.preventDefault();
    }
  });
}
