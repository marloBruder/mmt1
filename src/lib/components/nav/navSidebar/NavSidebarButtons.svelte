<script lang="ts">
  import { goto } from "$app/navigation";
  import { SettingsTab } from "$lib/components/tabs/SettingsTabComponent.svelte";
  import SettingsIcon from "$lib/icons/navSidebar/SettingsIcon.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import type { NavSidebarTabInfo } from "./NavSidebar.svelte";

  let { activeTab, isCollapsed, tabInfo, onClick = (i) => {} }: { activeTab: number; isCollapsed: boolean; tabInfo: NavSidebarTabInfo[]; onClick: (tabIndex: number) => void } = $props();

  let handleClick = $derived((tabIndex: number) => {
    onClick(tabIndex);
  });

  let settingsClick = () => {
    tabManager.openTab(new SettingsTab(), true);
  };
</script>

<div>
  {#each tabInfo as tab, index}
    <button class="w-12 h-12 overflow-hidden {activeTab == index && !isCollapsed ? 'text-gray-300' : 'text-gray-500'} inline-flex items-center justify-center" onclick={() => handleClick(index)} title={tab.title}><tab.icon /></button>
  {/each}
  <button class="w-12 h-12 overflow-hidden text-gray-500 inline-flex items-center justify-center" onclick={settingsClick} title="Settings"><SettingsIcon></SettingsIcon></button>
</div>
