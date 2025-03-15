<script lang="ts">
  import { TheoremTab } from "$lib/components/tabs/TheoremTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { label }: { label: string } = $props();

  let isOpenTab = $derived(tabManager.isSameTabOpen(new TheoremTab(label)));

  let explorerClick = () => {
    tabManager.openTab(new TheoremTab(label));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new TheoremTab(label));
  };
</script>

<ExplorerButton {isOpenTab} {explorerClick} {explorerDblClick}>{label}</ExplorerButton>
