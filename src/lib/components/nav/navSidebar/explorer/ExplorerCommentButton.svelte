<script lang="ts">
  import { CommentTab } from "$lib/components/tabs/CommentTabComponent.svelte";
  import type { HeaderPath } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerButton from "./ExplorerButton.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";

  let { headerPath, commentNum }: { headerPath: HeaderPath; commentNum: number } = $props();

  let newTab = () => {
    return new CommentTab(headerPath, commentNum);
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_comment", { headerPath, commentI: commentNum })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "comment-list-entry-id-" + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)), true);
  };
</script>

<ExplorerButton {newTab} {openInNewTheoremExplorer}>{"Comment " + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)}</ExplorerButton>
