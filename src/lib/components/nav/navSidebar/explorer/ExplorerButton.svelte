<script lang="ts">
  import type { Snippet } from "svelte";

  let { children, isOpenTab, explorerClick, explorerDblClick }: { children: Snippet; isOpenTab: boolean; explorerClick: () => void; explorerDblClick: () => void } = $props();
</script>

<div>
  <button class={"w-full text-left pl-2 " + (isOpenTab ? " custom-bg-active-color " : " custom-bg-hover-color ")} onclick={explorerClick} ondblclick={explorerDblClick}>
    {@render children()}
  </button>
</div>
