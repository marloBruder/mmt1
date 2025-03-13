<script lang="ts">
  import { FloatingHypothesisTab } from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { label }: { label: string } = $props();

  let isOpenTab = $derived(tabManager.isSameTabOpen(new FloatingHypothesisTab(label)));

  let explorerClick = () => {
    tabManager.openTab(new FloatingHypothesisTab(label));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new FloatingHypothesisTab(label));
  };
</script>

<ExplorerButton {isOpenTab} {explorerClick} {explorerDblClick}>{label}</ExplorerButton>
