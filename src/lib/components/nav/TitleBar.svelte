<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();

  let minimizeClick = () => {
    appWindow.minimize();
  };

  let maximizeClick = () => {
    appWindow.toggleMaximize();
  };

  let closeClick = () => {
    appWindow.close();
  };
</script>

<div class="h-8 w-screen flex justify-between" data-tauri-drag-region>
  <div class="pl-4 h-full flex items-center">
    <span class="text-xl">mmt1</span>
  </div>
  <div class="flex">
    <button class="mx-4" onclick={minimizeClick}>MIN</button>
    <button class="mx-4" onclick={maximizeClick}>MAX</button>
    <button class="mx-4" onclick={closeClick}>CLOSE</button>
  </div>
</div>
