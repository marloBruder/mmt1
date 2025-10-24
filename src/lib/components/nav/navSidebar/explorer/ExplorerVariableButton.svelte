<script lang="ts">
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { VariablesTab } from "$lib/components/tabs/VariablesTabComponent.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { save } from "@tauri-apps/plugin-dialog";

  let { variables }: { variables: string } = $props();

  let anyVariable = $derived(variables.split(" ")[0]);

  let newTab = () => {
    return new VariablesTab(anyVariable);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_variable", { anyVariable })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "variable-list-entry-id-" + anyVariable), true);
  };

  let turnIntoMmpFile = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Proof File", extensions: ["mmp"] }] });

    if (filePath) {
      await invoke("write_variable_mmp_format_to_file", { anyVariable, filePath });
    }
  };

  let copyMmpFormatToClipboard = async () => {
    let mmpFormat = (await invoke("get_variable_mmp_format", { anyVariable })) as string;

    navigator.clipboard.writeText(mmpFormat);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer} {turnIntoMmpFile} {copyMmpFormatToClipboard}>
  {"Variable" + (variables.includes(" ") ? "s" : "") + ": "}
  {#each variables.split(" ") as variable}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={variable}></MetamathExpression>
  {/each}
</ExplorerButton>
