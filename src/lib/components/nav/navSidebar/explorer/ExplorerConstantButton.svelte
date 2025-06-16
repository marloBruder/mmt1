<script lang="ts">
  import { ConstantsTab } from "$lib/components/tabs/ConstantsTabComponent.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { constants }: { constants: string } = $props();

  let anyConstant = $derived(constants.split(" ")[0]);

  let isOpenTab = $derived(tabManager.isSameTabOpen(new ConstantsTab(anyConstant)));

  let explorerClick = () => {
    tabManager.openTab(new ConstantsTab(anyConstant));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new ConstantsTab(anyConstant));
  };
</script>

<ExplorerButton {isOpenTab} {explorerClick} {explorerDblClick}>
  {"Constant" + (constants.includes(" ") ? "s" : "") + ": "}
  {#each constants.split(" ") as constant}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={constant}></MetamathExpression>
  {/each}
</ExplorerButton>
