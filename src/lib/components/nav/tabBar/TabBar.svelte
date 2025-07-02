<script lang="ts">
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
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

  let splitClick = async () => {
    await tabManager.getOpenTab()?.split();
  };
</script>

<div class="h-8 flex flex-nowrap border-b border-gray-300 overflow-hidden">
  <div class="h-full flex flex-nowrap border-r border-gray-300">
    <button onclick={previousClick} class={tabManager.getOpenTab()?.previousTab ? "text-gray-700 " : "text-gray-400 "}><ArrowLeftIcon></ArrowLeftIcon></button>
    <button onclick={nextClick} class={tabManager.getOpenTab()?.nextTab ? "text-gray-700 " : "text-gray-400 "}><ArrowRightIcon></ArrowRightIcon></button>
  </div>
  <ScrollableContainer theme="os-custom-scrollbar-theme-small">
    <div class="h-full flex-grow flex flex-nowrap">
      {#each tabManager.tabs as tab, index}
        <TabBarItem {tab} {index}></TabBarItem>
      {/each}
    </div>
  </ScrollableContainer>
  {#if tabManager.getOpenTab() != null && !tabManager.getOpenTab()!.splitDisabled()}
    <div class="h-full flex flex-nowrap border-l border-gray-300">
      <button class="px-2" onclick={splitClick}>SPLIT</button>
    </div>
  {/if}
</div>
