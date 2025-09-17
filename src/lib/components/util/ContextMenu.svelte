<script lang="ts" module>
  export type ContextMenuContext = "fileButton";

  class ContextMenuData {
    visible = $state(false);
    context: ContextMenuContext = $state("fileButton");
    x = $state(0);
    y = $state(0);
  }

  let contextMenuData = new ContextMenuData();

  export { contextMenuData };
</script>

<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  let handleClickOutside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById("context-menu");
    if (contextMenuDiv && !contextMenuDiv.contains(e.target as Node)) {
      contextMenuData.visible = false;
    }
  };

  let handleClickInside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById("context-menu");
    let target = e.target;
    if (target instanceof HTMLButtonElement && contextMenuDiv && contextMenuDiv.contains(target as Node)) {
      contextMenuData.visible = false;
    }
  };

  let handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && contextMenuData.visible) {
      contextMenuData.visible = false;
    }
  };

  $effect(() => {
    if (contextMenuData.visible) {
      let contextMenuDiv = document.getElementById("context-menu");
      if (contextMenuDiv !== null) {
        contextMenuDiv.style.left = contextMenuData.x + "px";
        contextMenuDiv.style.top = contextMenuData.y + "px";
      }
    }
  });

  onMount(() => {
    document.addEventListener("click", handleClickInside);
    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickInside);
    document.removeEventListener("mousedown", handleClickOutside);
    document.removeEventListener("keydown", handleKeydown);
  });
</script>

{#if contextMenuData.visible}
  <div id="context-menu" class="fixed top-0 left-0 custom-bg-dropdown-color z-50 text-gray-300 py-2 rounded-lg">
    {#if contextMenuData.context === "fileButton"}
      <div>
        <button>lallalalalal</button>
      </div>
      <div>
        <button>lallalalalal</button>
      </div>
      <div>
        <button>lallalalalal</button>
      </div>
      <div>
        <button>lallalalalal</button>
      </div>
      <div>
        <button>lallalalalal</button>
      </div>
    {/if}
  </div>
{/if}
