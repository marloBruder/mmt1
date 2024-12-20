<script lang="ts">
  import { theoremData } from "$lib/sharedState/metamathData/theoremData.svelte";
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";

  let explorerClick = (name: string) => {
    tabManager.addTabAndOpen(new TheoremTab(name));
  };
</script>

<div>
  <div class="py-2">Explorer:</div>
  {#each theoremData.theorems as theorem}
    <div>
      <button onclick={() => explorerClick(theorem.name)}>{theorem.name}</button>
    </div>
  {/each}
</div>
