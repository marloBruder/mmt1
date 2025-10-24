<script lang="ts">
  import { FloatingHypothesisTab } from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { save } from "@tauri-apps/plugin-dialog";

  let { label }: { label: string } = $props();

  let newTab = () => {
    return new FloatingHypothesisTab(label);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_floating_hypothesis", { floatingHypothesisLabel: label })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "floating-hypothesis-list-entry-id-" + label), true);
  };

  let turnIntoMmpFile = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Proof File", extensions: ["mmp"] }] });

    if (filePath) {
      await invoke("write_floating_hypothesis_mmp_format_to_file", { label, filePath });
    }
  };

  let copyMmpFormatToClipboard = async () => {
    let mmpFormat = (await invoke("get_floating_hypothesis_mmp_format", { label })) as string;

    navigator.clipboard.writeText(mmpFormat);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer} {turnIntoMmpFile} {copyMmpFormatToClipboard}>{label}</ExplorerButton>
