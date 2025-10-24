<script lang="ts">
  import { CommentTab } from "$lib/components/tabs/CommentTabComponent.svelte";
  import type { HeaderPath } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { save } from "@tauri-apps/plugin-dialog";

  let { headerPath, commentNum }: { headerPath: HeaderPath; commentNum: number } = $props();

  let newTab = () => {
    return new CommentTab(headerPath, commentNum);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_comment", { headerPath, commentI: commentNum })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "comment-list-entry-id-" + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)), true);
  };

  let turnIntoMmpFile = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Proof File", extensions: ["mmp"] }] });

    if (filePath) {
      await invoke("write_comment_mmp_format_to_file", { headerPath, commentI: commentNum, filePath });
    }
  };

  let copyMmpFormatToClipboard = async () => {
    let mmpFormat = (await invoke("get_comment_mmp_format", { headerPath, commentI: commentNum })) as string;

    navigator.clipboard.writeText(mmpFormat);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer} {turnIntoMmpFile} {copyMmpFormatToClipboard}>{"Comment " + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)}</ExplorerButton>
