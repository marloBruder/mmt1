<script lang="ts">
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { TheoremTab } from "$lib/components/tabs/TheoremTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { label }: { label: string } = $props();

  let newTab = () => {
    return new TheoremTab(label);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_theorem", { theoremLabel: label })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "theorem-list-entry-id-" + label), true);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer}>{label}</ExplorerButton>
