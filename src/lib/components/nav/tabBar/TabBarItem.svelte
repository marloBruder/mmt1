<script lang="ts">
  import { Tab, tabManager } from "$lib/sharedState/tabManager.svelte";

  let { tab, index }: { tab: Tab; index: number } = $props();

  let tabClick = (index: number) => {
    tabManager.openTabWithIndex(index);
  };

  let tabClose = async (index: number) => {
    await tabManager.closeTabWithIndex(index);
  };

  let tabDblClick = (index: number) => {
    tabManager.makeTempTabWithIndexPermanent(index);
  };

  let tabCloseHover = $state(false);

  let tabCloseMouseEnter = () => {
    tabCloseHover = true;
  };

  let tabCloseMouseLeave = () => {
    tabCloseHover = false;
  };
</script>

<div class={"whitespace-nowrap " + (index == tabManager.openTabIndex ? "border-b-2 border-gray-400 " : "")}>
  <button onclick={() => tabClick(index)} ondblclick={() => tabDblClick(index)} class={"h-full px-2 " + (index == tabManager.tempTabIndex ? "italic " : "")}>{tab.name()}</button>
  <button onclick={() => tabClose(index)} onmouseenter={tabCloseMouseEnter} onmouseleave={tabCloseMouseLeave} class="h-full px-2 border-r border-gray-400 -ml-1">
    {#if !tab.showDot() || tabCloseHover}
      X
    {:else}
      •
    {/if}
  </button>
</div>
