<script lang="ts">
  import { tabManager, TheoremExplorerTab, TheoremTab, type Tab } from "$lib/sharedState/tabData.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";
  import RoundButton from "../util/RoundButton.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremExplorerTab = $derived.by(() => {
    if (tab instanceof TheoremExplorerTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let theoremClick = async (label: string) => {
    await tabManager.changeTab(new TheoremTab(label));
  };

  let nextPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start + 100);
  };

  let previousPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start - 100);
  };
</script>

{#each theoremExplorerTab.theoremList as theoremListEntry}
  <div class="my-10 text-center border-black border-y">
    <div>
      <button onclick={() => theoremClick(theoremListEntry.name)}>
        {theoremListEntry.name}
      </button>
      <small>
        {theoremListEntry.theoremNumber}
      </small>
    </div>
    <div class="border-gray-500 border-y">
      {theoremListEntry.description}
    </div>
    {#each theoremListEntry.hypotheses as hypothesis}
      <div>
        <MetamathExpression expression={hypothesis}></MetamathExpression>
      </div>
    {/each}
    <div class={theoremListEntry.hypotheses.length != 0 ? "border-t border-gray-400 " : ""}>
      <MetamathExpression expression={theoremListEntry.assertion}></MetamathExpression>
    </div>
  </div>
{/each}
<div class=" p-2 flex justify-around">
  <RoundButton onclick={previousPageClick}>Previous Page</RoundButton>
  <RoundButton onclick={nextPageClick}>Next Page</RoundButton>
</div>
