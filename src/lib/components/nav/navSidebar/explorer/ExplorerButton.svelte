<script lang="ts">
  import { FloatingHypothesisTab } from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import type { HeaderContentRepresentation } from "$lib/sharedState/model.svelte";
  import { tabManager, TheoremTab } from "$lib/sharedState/tabManager.svelte";

  let { contentTitle }: { contentTitle: HeaderContentRepresentation } = $props();

  let isOpenTab = $derived.by(() => {
    let openTab = tabManager.getOpenTab();
    return (contentTitle.contentType == "TheoremStatement" && openTab instanceof TheoremTab && openTab.theoremName == contentTitle.title) || (contentTitle.contentType == "FloatingHypohesisStatement" && openTab instanceof FloatingHypothesisTab && openTab.label == contentTitle.title);
  });

  let explorerClick = () => {
    switch (contentTitle.contentType) {
      case "FloatingHypohesisStatement":
        tabManager.openTab(new FloatingHypothesisTab(contentTitle.title));
        break;
      case "TheoremStatement":
        tabManager.openTab(new TheoremTab(contentTitle.title));
        break;
    }
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new TheoremTab(contentTitle.title));
  };
</script>

<div>
  <button class={"w-full text-left pl-2 " + (isOpenTab ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={() => explorerClick()} ondblclick={() => explorerDblClick()}>
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
