<script lang="ts">
  import { VariablesTab } from "$lib/components/tabs/VariablesTabComponent.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { variables }: { variables: string } = $props();

  let anyVariable = $derived(variables.split(" ")[0]);

  let isOpenTab = $derived(tabManager.isSameTabOpen(new VariablesTab(anyVariable)));

  let explorerClick = () => {
    tabManager.openTab(new VariablesTab(anyVariable));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new VariablesTab(anyVariable));
  };
</script>

<ExplorerButton {isOpenTab} {explorerClick} {explorerDblClick}>
  {"Variable" + (variables.includes(" ") ? "s" : "") + ": "}
  {#each variables.split(" ") as variable}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={variable}></MetamathExpression>
  {/each}
</ExplorerButton>
