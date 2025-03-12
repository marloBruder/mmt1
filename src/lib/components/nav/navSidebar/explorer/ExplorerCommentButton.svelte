<script lang="ts">
  import { CommentTab } from "$lib/components/tabs/CommentTabComponent.svelte";
  import type { HeaderPath } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { util } from "$lib/sharedState/util.svelte";

  let { headerPath, commentNum }: { headerPath: HeaderPath; commentNum: number } = $props();

  let isOpenTab = $derived(tabManager.doesSameTabExist(new CommentTab(headerPath, commentNum)));

  let explorerClick = () => {
    tabManager.openTab(new CommentTab(headerPath, commentNum));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new CommentTab(headerPath, commentNum));
  };
</script>

<div>
  <button class={"w-full text-left pl-2 " + (isOpenTab ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={explorerClick} ondblclick={explorerDblClick}>{"Comment " + util.headerPathToStringRep(headerPath) + "#" + (commentNum + 1)}</button>
</div>
