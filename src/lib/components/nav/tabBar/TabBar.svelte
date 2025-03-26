<script lang="ts">
  import ArrowLeftIcon from "$lib/icons/ArrowLeftIcon.svelte";
  import ArrowRightIcon from "$lib/icons/ArrowRightIcon.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import TabBarItem from "./TabBarItem.svelte";

  let previousClick = async () => {
    await tabManager.switchToPreviousTab();
  };
  let nextClick = async () => {
    await tabManager.switchToNextTab();
  };
</script>

<div class="h-8 flex felx-nowrap border-b border-gray-400 overflow-hidden">
  <div class="h-full flex flex-nowrap border-r border-gray-400">
    <button onclick={previousClick} class={tabManager.getOpenTab()?.previousTab ? "text-gray-700 " : "text-gray-400 "}><ArrowLeftIcon></ArrowLeftIcon></button>
    <button onclick={nextClick} class={tabManager.getOpenTab()?.nextTab ? "text-gray-700 " : "text-gray-400 "}><ArrowRightIcon></ArrowRightIcon></button>
  </div>
  <div class="h-full flex flex-nowrap">
    {#each tabManager.tabs as tab, index}
      <TabBarItem {tab} {index}></TabBarItem>
    {/each}
  </div>
</div>
