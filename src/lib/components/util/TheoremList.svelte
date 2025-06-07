<script lang="ts">
  import type { TheoremListData } from "$lib/sharedState/model.svelte";
  import MetamathExpression from "./MetamathExpression.svelte";
  import RoundButton from "./RoundButton.svelte";
  import TheoremLink from "./TheoremLink.svelte";

  let { theoremListData, previousPageClick, nextPageClick, pageButtonClick, pageNum }: { theoremListData: TheoremListData; previousPageClick: () => void; nextPageClick: () => void; pageButtonClick: (pageNum: number) => void; pageNum: number } = $props();
</script>

{#each theoremListData.list as theoremListEntry}
  <div class="my-10 text-center border-black border-y">
    {#if theoremListEntry.discriminator === "TheoremListEntry"}
      <div>
        <TheoremLink label={theoremListEntry.label}></TheoremLink>
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
    {:else if theoremListEntry.discriminator === "HeaderListEntry"}
      <div class="text-2xl p-4">
        {theoremListEntry.headerPath + " " + theoremListEntry.title}
      </div>
    {:else if theoremListEntry.discriminator === "CommentListEntry"}
      <div>
        <div class="text-xl">
          Comment {theoremListEntry.commentPath}
        </div>
        <div>
          {theoremListEntry.text}
        </div>
      </div>
    {/if}
  </div>
{/each}
<div class=" p-2 flex justify-around">
  <RoundButton onclick={previousPageClick} disabled={pageNum <= 0}>Previous Page</RoundButton>
  <RoundButton onclick={nextPageClick} disabled={pageNum >= theoremListData.pageAmount - 1}>Next Page</RoundButton>
</div>
<div>
  {#each { length: theoremListData.pageAmount } as _, i}
    <button onclick={() => pageButtonClick(i)} class="px-2">{i + 1}</button>
  {/each}
</div>
