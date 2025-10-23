<script lang="ts">
  import { FloatingHypothesisTab } from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";

  let { label }: { label: string } = $props();

  let newTab = () => {
    return new FloatingHypothesisTab(label);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_floating_hypothesis", { floatingHypothesisLabel: label })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "floating-hypothesis-list-entry-id-" + label), true);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer}>{label}</ExplorerButton>
