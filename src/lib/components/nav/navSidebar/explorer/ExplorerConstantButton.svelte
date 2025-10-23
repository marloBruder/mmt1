<script lang="ts">
  import { ConstantsTab } from "$lib/components/tabs/ConstantsTabComponent.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { constants }: { constants: string } = $props();

  let anyConstant = $derived(constants.split(" ")[0]);

  let newTab = () => {
    return new ConstantsTab(anyConstant);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_constant", { anyConstant })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "constant-list-entry-id-" + anyConstant), true);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer}>
  {"Constant" + (constants.includes(" ") ? "s" : "") + ": "}
  {#each constants.split(" ") as constant}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={constant}></MetamathExpression>
  {/each}
</ExplorerButton>
