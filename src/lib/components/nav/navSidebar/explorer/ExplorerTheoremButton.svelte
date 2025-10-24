<script lang="ts">
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { TheoremTab } from "$lib/components/tabs/TheoremTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { save } from "@tauri-apps/plugin-dialog";

  let { label }: { label: string } = $props();

  let newTab = () => {
    return new TheoremTab(label);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_theorem", { theoremLabel: label })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "theorem-list-entry-id-" + label), true);
  };

  let turnIntoMmpFile = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Proof File", extensions: ["mmp"] }] });

    if (filePath) {
      await invoke("write_theorem_mmp_format_to_file", { label, filePath });
    }
  };

  let copyMmpFormatToClipboard = async () => {
    let mmpFormat = (await invoke("get_theorem_mmp_format", { label })) as string;

    navigator.clipboard.writeText(mmpFormat);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer} {turnIntoMmpFile} {copyMmpFormatToClipboard}>{label}</ExplorerButton>
