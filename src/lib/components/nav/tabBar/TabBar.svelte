<script lang="ts">
  import Dropdown from "$lib/components/util/Dropdown.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import ArrowLeftIcon from "$lib/icons/ArrowLeftIcon.svelte";
  import ArrowRightIcon from "$lib/icons/ArrowRightIcon.svelte";
  import SplitViewIcon from "$lib/icons/SplitViewIcon.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import TabBarItem from "./TabBarItem.svelte";

  let previousClick = async () => {
    await tabManager.switchToPreviousTab();
  };
  let nextClick = async () => {
    await tabManager.switchToNextTab();
  };

  let splitNoneClick = async () => {
    tabManager.setSplitTabState("none");
  };
  let splitVerticalClick = async () => {
    tabManager.setSplitTabState("splitVertical");
  };
  let splitHorizontalClick = async () => {
    tabManager.setSplitTabState("splitHorizontal");
  };
  let splitExternalWindowClick = async () => {
    tabManager.setSplitTabState("externalWindow");
  };
</script>

<div class="h-8 flex flex-nowrap border-b border-gray-300 overflow-hidden">
  <div class="h-full flex flex-nowrap border-r border-gray-300">
    <button onclick={previousClick} class={tabManager.getOpenTab()?.previousTab ? "" : "text-gray-700 "}><ArrowLeftIcon></ArrowLeftIcon></button>
    <button onclick={nextClick} class={tabManager.getOpenTab()?.nextTab ? "" : "text-gray-700 "}><ArrowRightIcon></ArrowRightIcon></button>
  </div>
  <ScrollableContainer horizontalScroll theme="os-custom-scrollbar-theme-small">
    <div class="h-full flex-grow flex flex-nowrap">
      {#each tabManager.tabs as tab, index}
        <TabBarItem {tab} {index}></TabBarItem>
      {/each}
    </div>
  </ScrollableContainer>
  <div class="h-full flex flex-nowrap items-center border-l border-gray-300">
    <div class="px-1 h-6">
      <Dropdown title="Split view" alignDropdownLeft>
        {#snippet buttonContent()}
          <SplitViewIcon />
        {/snippet}
        {#snippet dropdownContent()}
          <div class="px-2 {tabManager.splitTabState === 'none' ? 'bg-purple-500' : ''}">
            <button onclick={splitNoneClick}>Don't Split Editor Tabs</button>
          </div>
          <div class="px-2 {tabManager.splitTabState === 'splitVertical' ? 'bg-purple-500' : ''}">
            <button onclick={splitVerticalClick}>Split Editor Tabs Vertically</button>
          </div>
          <div class="px-2 {tabManager.splitTabState === 'splitHorizontal' ? 'bg-purple-500' : ''}">
            <button onclick={splitHorizontalClick}>Split Editor Tabs Horizontally</button>
          </div>
          <div class="px-2 {tabManager.splitTabState === 'externalWindow' ? 'bg-purple-500' : ''}">
            <button onclick={splitExternalWindowClick}>Open In External Window</button>
          </div>
        {/snippet}
      </Dropdown>
    </div>
  </div>
</div>
