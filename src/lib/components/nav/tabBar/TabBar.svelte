<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import TabBarItem from "./TabBarItem.svelte";
</script>

<div class="h-8 flex flex-nowrap border-b border-gray-400 overflow-hidden">
  {#each tabManager.tabs as tab, index}
    <TabBarItem {tab} {index}></TabBarItem>
  {/each}
</div>
