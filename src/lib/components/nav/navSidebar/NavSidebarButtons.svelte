<script lang="ts">
  import { goto } from "$app/navigation";
  import { SettingsTab, tabManager } from "$lib/sharedState/tabData.svelte";

  let { activeTab, tabInfo, onClick = (i) => {} }: { activeTab: number; tabInfo: { title: string }[]; onClick: (tabIndex: number) => void } = $props();

  let handleClick = $derived((tabIndex: number) => {
    onClick(tabIndex);
  });

  let settingsClick = () => {
    tabManager.openTab(new SettingsTab(), true);
  };
</script>

<div>
  {#each tabInfo as tab, index}
    <button class="w-12 h-12 overflow-hidden {activeTab == index ? 'font-bold' : ''}" onclick={() => handleClick(index)} title={tab.title}>Tab {index + 1}</button>
  {/each}
  <button class="w-12 h-12 overflow-hidden" onclick={settingsClick} title="Settings">Tab 4</button>
  <button
    class="w-12 h-12 overflow-hidden"
    title="Main Menu"
    onclick={() => {
      goto("/");
    }}>Main</button
  >
</div>
