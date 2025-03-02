<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";

  let tabClick = (index: number) => {
    tabManager.openTabWithIndex(index);
  };

  let tabClose = (index: number) => {
    tabManager.closeTabWithIndex(index);
  };

  let tabDblClick = (index: number) => {
    tabManager.makeTempTabWithIndexPermanent(index);
  };
</script>

<div class="h-8 flex flex-nowrap border-b border-gray-400 overflow-hidden">
  {#each tabManager.tabs as tab, index}
    <div class={"whitespace-nowrap " + (index == tabManager.openTabIndex ? "border-b-2 border-gray-400 " : "")}>
      <button onclick={() => tabClick(index)} ondblclick={() => tabDblClick(index)} class={"h-full px-2 " + (index == tabManager.tempTabIndex ? "italic " : "")}>{tab.name()}</button>
      <button onclick={() => tabClose(index)} class="h-full px-2 border-r border-gray-400 -ml-1">X</button>
    </div>
  {/each}
</div>
