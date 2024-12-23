<script lang="ts">
  import { goto } from "$app/navigation";

  let { activeTab, tabInfo, onClick = (i) => {} }: { activeTab: number; tabInfo: { title: string }[]; onClick: (tabIndex: number) => void } = $props();

  let handleClick = $derived((tabIndex: number) => {
    onClick(tabIndex);
  });

  let openSettings = () => {
    goto("/main/settings");
  };
</script>

<div>
  {#each tabInfo as tab, index}
    <button class="w-12 h-12 overflow-hidden {activeTab == index ? 'font-bold' : ''}" onclick={() => handleClick(index)} title={tab.title}>Tab {index + 1}</button>
  {/each}
  <button class="w-12 h-12 overflow-hidden" onclick={openSettings} title="Settings">Tab 4</button>
</div>
