<script lang="ts">
  let { activeTab, tabInfo, onClick = (i) => {} }: { activeTab: number; tabInfo: { title: string }[]; onClick: (tabIndex: number) => void } = $props();

  let handleClick = $derived((tabIndex: number) => {
    return () => {
      onClick(tabIndex);
    };
  });
</script>

<div>
  {#each tabInfo as tab, index}
    <button class="w-12 h-12 overflow-hidden {activeTab == index ? 'font-bold' : ''}" onclick={handleClick(index)} title={tab.title}>Tab {index}</button>
  {/each}
</div>
