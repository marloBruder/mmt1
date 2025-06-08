<script lang="ts">
  import type { TheoremListData } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { CommentTab } from "../tabs/CommentTabComponent.svelte";
  import { FloatingHypothesisTab } from "../tabs/FloatingHypothesisTabComponent.svelte";
  import MetamathExpression from "./MetamathExpression.svelte";
  import RoundButton from "./RoundButton.svelte";
  import TheoremLink from "./TheoremLink.svelte";

  let { theoremListData, previousPageClick, nextPageClick, pageButtonClick, pageNum }: { theoremListData: TheoremListData; previousPageClick: () => void; nextPageClick: () => void; pageButtonClick: (pageNum: number) => void; pageNum: number } = $props();

  let commentClick = (commentPath: string) => {
    let [headerPathString, commentNumString] = commentPath.split("#");
    let commentNum = Number(commentNumString) - 1;
    let headerPath =
      headerPathString === ""
        ? { path: [] }
        : {
            path: headerPathString.split(".").map((pathSeg) => Number(pathSeg) - 1),
          };
    tabManager.changeTab(new CommentTab(headerPath, commentNum));
  };

  let floatingHypothesisClick = (label: string) => {
    tabManager.changeTab(new FloatingHypothesisTab(label));
  };
</script>

{#each theoremListData.list as theoremListEntry}
  <div class="my-10 text-center border-black border-y">
    {#if theoremListEntry.discriminator === "HeaderListEntry"}
      <div class="text-2xl p-4">
        {theoremListEntry.headerPath + " " + theoremListEntry.title}
      </div>
    {:else if theoremListEntry.discriminator === "CommentListEntry"}
      <div>
        <div class="text-xl">
          <button onclick={() => commentClick(theoremListEntry.commentPath)}>{theoremListEntry.commentPath}</button>
        </div>
        <div>
          {theoremListEntry.text}
        </div>
      </div>
    {:else if theoremListEntry.discriminator === "ConstantListEntry"}
      <div>
        <div class="text-xl">Constants:</div>
        <div>
          <MetamathExpression expression={theoremListEntry.constants}></MetamathExpression>
        </div>
      </div>
    {:else if theoremListEntry.discriminator === "VariableListEntry"}
      <div>
        <div class="text-xl">Variables:</div>
        <div>
          <MetamathExpression expression={theoremListEntry.variables}></MetamathExpression>
        </div>
      </div>
    {:else if theoremListEntry.discriminator === "FloatingHypothesisListEntry"}
      <div>
        <div><button onclick={() => floatingHypothesisClick(theoremListEntry.label)}>{theoremListEntry.label}</button></div>
        <div>
          <MetamathExpression expression={theoremListEntry.typecode + " " + theoremListEntry.variable}></MetamathExpression>
        </div>
      </div>
    {:else if theoremListEntry.discriminator === "TheoremListEntry"}
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
