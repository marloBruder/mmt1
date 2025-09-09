import { confirm } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";

export function setupLinkIntercepter() {
  document.addEventListener(
    "click",
    async (e: MouseEvent) => {
      console.log("test1");
      const target = e.target as Element | null;
      const a = target?.closest?.("a[href]") as HTMLAnchorElement | null;
      if (!a) return;
      const url = a.href;
      e.preventDefault();
      e.stopImmediatePropagation();
      console.log("test2");
      if (await confirm("Are you sure you want to open the link '" + url + "' in an external browser?")) {
        openUrl(url);
      }
    },
    true
  );
}
