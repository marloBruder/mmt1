<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { onDestroy, onMount, type Snippet } from "svelte";
  import { contextMenuData, type ContextMenuContext } from "./ContextMenu.svelte";

  let { children, context }: { children: Snippet; context: ContextMenuContext } = $props();

  let oncontextmenu = (e: MouseEvent) => {
    contextMenuData.visible = true;
    contextMenuData.context = context;
    contextMenuData.x = e.clientX;
    contextMenuData.y = e.clientY;
  };
</script>

<div role="none" {oncontextmenu}>
  {@render children()}
</div>
