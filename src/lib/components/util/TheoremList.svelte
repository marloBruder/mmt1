<script lang="ts">
  import type { TheoremListEntry } from "$lib/sharedState/model.svelte";
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";
  import type { MouseEventHandler } from "svelte/elements";
  import MetamathExpression from "./MetamathExpression.svelte";
  import RoundButton from "./RoundButton.svelte";

  let { theoremList, previousPageClick = () => {}, nextPageClick = () => {} }: { theoremList: TheoremListEntry[]; previousPageClick?: MouseEventHandler<HTMLButtonElement>; nextPageClick?: MouseEventHandler<HTMLButtonElement> } = $props();

  let theoremClick = async (label: string) => {
    await tabManager.changeTab(new TheoremTab(label));
  };
</script>

{#each theoremList as theoremListEntry}
  <div class="my-10 text-center border-black border-y">
    <div>
      <button onclick={() => theoremClick(theoremListEntry.label)}>
        {theoremListEntry.label}
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
