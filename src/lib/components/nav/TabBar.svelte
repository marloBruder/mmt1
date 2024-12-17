<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabData.svelte";

  let tabClick = (index: number) => {
    return () => {
      tabManager.openTab(index);
    };
  };

  let tabClose = (index: number) => {
    return () => {
      tabManager.closeTab(index);
    };
  };
</script>

<div class="h-8 flex flex-nowrap border-b border-gray-400 overflow-hidden">
  {#each tabManager.tabs as tab, index}
    <div class="whitespace-nowrap">
      <button onclick={tabClick(index)} class="h-full px-2">{tab.name()}</button>
      <button onclick={tabClose(index)} class="h-full px-2 border-r border-gray-400 -ml-1">X</button>
    </div>
  {/each}
</div>
