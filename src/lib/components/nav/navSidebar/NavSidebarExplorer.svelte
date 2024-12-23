<script lang="ts">
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";

  let explorerClick = (name: string) => {
    tabManager.addTabAndOpen(new TheoremTab(name));
  };
</script>

<div>
  <div class="py-2">Explorer:</div>
  {#each nameListData.theoremNames as name}
    <div>
      <button onclick={() => explorerClick(name)}>{name}</button>
    </div>
  {/each}
</div>
