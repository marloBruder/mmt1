<script lang="ts">
  import { CommentTab } from "$lib/components/tabs/CommentTabComponent.svelte";
  import type { HeaderPath } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { headerPath, commentNum }: { headerPath: HeaderPath; commentNum: number } = $props();

  let isOpenTab = $derived(tabManager.isSameTabOpen(new CommentTab(headerPath, commentNum)));

  let explorerClick = () => {
    tabManager.openTab(new CommentTab(headerPath, commentNum));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new CommentTab(headerPath, commentNum));
  };
</script>

<ExplorerButton {isOpenTab} {explorerClick} {explorerDblClick}>{"Comment " + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)}</ExplorerButton>
