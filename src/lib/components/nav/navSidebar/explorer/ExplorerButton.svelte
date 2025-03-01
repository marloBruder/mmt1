<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import type { HeaderContentRepresentation } from "$lib/sharedState/model.svelte";
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";

  let { contentTitle, openTheoremName }: { contentTitle: HeaderContentRepresentation; openTheoremName: string | null } = $props();

  let explorerClick = () => {
    tabManager.openTab(new TheoremTab(contentTitle.title));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new TheoremTab(contentTitle.title));
  };
</script>

<div>
  <button class={"w-full text-left pl-2 " + (contentTitle.title === openTheoremName ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={() => explorerClick()} ondblclick={() => explorerDblClick()}>
    {#if contentTitle.contentType === "ConstantStatement"}
      {"Constant: "}
      <MetamathExpression expression={contentTitle.title}></MetamathExpression>
    {:else if contentTitle.contentType === "VariableStatement"}
      {"Variable: "}
      <MetamathExpression expression={contentTitle.title}></MetamathExpression>
    {:else}
      {contentTitle.title}
    {/if}
  </button>
</div>
