<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { onDestroy, onMount, type Snippet } from "svelte";

  let { element, contextMenu }: { element: Snippet; contextMenu: Snippet } = $props();

  let contextMenuId = "context-menu-id-" + createInstanceId();

  let contextMenuVisible = $state(false);
  let lastMouseX = $state(0);
  let lastMouseY = $state(0);

  let oncontextmenu = (e: MouseEvent) => {
    contextMenuVisible = true;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  };

  let handleClickOutside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById(contextMenuId);
    if (contextMenuDiv && !contextMenuDiv.contains(e.target as Node)) {
      contextMenuVisible = false;
    }
  };

  let handleClickInside = (e: MouseEvent) => {
    let contextMenuDiv = document.getElementById(contextMenuId);
    let target = e.target;
    if (target instanceof HTMLButtonElement && contextMenuDiv && contextMenuDiv.contains(target as Node)) {
      contextMenuVisible = false;
    }
  };

  let handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && contextMenuVisible) {
      contextMenuVisible = false;
    }
  };

  $effect(() => {
    if (contextMenuVisible) {
      let contextMenuDiv = document.getElementById(contextMenuId);
      if (contextMenuDiv !== null) {
        contextMenuDiv.focus();
        contextMenuDiv.style.left = lastMouseX + "px";
        contextMenuDiv.style.top = lastMouseY + "px";
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

<div role="none" {oncontextmenu}>
  {@render element()}
</div>
{#if contextMenuVisible}
  <div role="none" id={contextMenuId} class="fixed top-0 left-0 custom-bg-dropdown-color custom-z-index">
    {@render contextMenu()}
  </div>
{/if}

<style>
  .custom-z-index {
    z-index: 9999;
  }
</style>
