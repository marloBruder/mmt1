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

  function portal(node: HTMLElement, targetSelector = "body") {
    let target: Element | null = null;

    onMount(() => {
      target = document.querySelector(targetSelector);
      if (!target) {
        return;
      }

      const originalParent = node.parentNode;
      const anchor = document.createComment("svelte-portal-anchor");
      if (originalParent) {
        originalParent.insertBefore(anchor, node);
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
