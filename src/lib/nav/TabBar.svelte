<script>
  import { tabManager } from "$lib/sharedState/tabData.svelte";
</script>

<div class="h-8 border-b border-gray-400">
  {#each tabManager.tabs as tab, index}
    <button
      onclick={() => {
        tabManager.openTab(index);
      }}
      class="h-full px-2 border-r border-gray-400">{tab.name()}</button
    >
  {/each}
</div>
