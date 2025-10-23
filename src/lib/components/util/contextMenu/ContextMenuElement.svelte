<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { onDestroy, onMount, type Snippet } from "svelte";

  let { element, contextMenu }: { element: Snippet; contextMenu: Snippet } = $props();

  let contextMenuId = "context-menu-id-" + createInstanceId();
  let dummyContextMenuId = "dummy-context-menu-id-" + createInstanceId();

  let contextMenuVisible = $state(false);

  let nextContextMenuLeft = $state(0);
  let nextContextMenuTop = $state(0);

  let dummyContextMenuVisible = $state(false);

  let oncontextmenu = (e: MouseEvent) => {
    dummyContextMenuVisible = true;
    let lastMouseX = e.clientX;
    let lastMouseY = e.clientY;

    requestAnimationFrame(() => {
      let dummyContextMenu = document.getElementById(dummyContextMenuId);
      if (dummyContextMenu !== null) {
        if (lastMouseX + dummyContextMenu.clientWidth > window.innerWidth) {
          nextContextMenuLeft = lastMouseX - dummyContextMenu.clientWidth;
        } else {
          nextContextMenuLeft = lastMouseX;
        }

        if (lastMouseY + dummyContextMenu.clientHeight > window.innerHeight) {
          nextContextMenuTop = lastMouseY - dummyContextMenu.clientHeight;
        } else {
          nextContextMenuTop = lastMouseY;
        }

        contextMenuVisible = true;
      }
    });
  };

  let handleClickOutside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById(contextMenuId);
    if (contextMenuDiv && !contextMenuDiv.contains(e.target as Node)) {
      dummyContextMenuVisible = false;
      contextMenuVisible = false;
    }
  };

  let handleClickInside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById(contextMenuId);
    let target = e.target;
    if (target instanceof HTMLButtonElement && contextMenuDiv && contextMenuDiv.contains(target as Node)) {
      dummyContextMenuVisible = false;
      contextMenuVisible = false;
    }
  };

  let handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && contextMenuVisible) {
      dummyContextMenuVisible = false;
      contextMenuVisible = false;
    }
  };

  $effect(() => {
    if (contextMenuVisible) {
      let contextMenuDiv = document.getElementById(contextMenuId);
      if (contextMenuDiv !== null) {
        contextMenuDiv.focus();
        contextMenuDiv.style.left = nextContextMenuLeft + "px";
        contextMenuDiv.style.top = nextContextMenuTop + "px";
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

  function portal(node: HTMLElement) {
    let target: Element | null = null;

    onMount(() => {
      target = document.querySelector("body");

      if (!target) {
        return;
      }

      target.appendChild(node);
    });

    onDestroy(() => {
      if (target && target.contains(node)) {
        target.removeChild(node);
      }
    });

    return {
      destroy() {},
    };
  }
</script>

<div role="none" {oncontextmenu}>
  {@render element()}
</div>
{#if contextMenuVisible}
  <div use:portal id={contextMenuId} class="fixed custom-bg-dropdown-color z-50 text-gray-300 py-2 rounded-lg">
    {@render contextMenu()}
  </div>
{/if}
{#if dummyContextMenuVisible}
  <div id={dummyContextMenuId} class="fixed py-2 rounded-lg custom-off-screen invisible pointer-events-none">
    {@render contextMenu()}
  </div>
{/if}

<style>
  .custom-off-screen {
    left: -9999px;
    top: -9999px;
  }
</style>
