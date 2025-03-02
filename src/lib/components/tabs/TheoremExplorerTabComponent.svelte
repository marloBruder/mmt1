<script lang="ts">
  import { TheoremExplorerTab, type Tab } from "$lib/sharedState/tabManager.svelte";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremExplorerTab = $derived.by(() => {
    if (tab instanceof TheoremExplorerTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let nextPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start + 100);
  };

  let previousPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start - 100);
  };
</script>

<TheoremList theoremList={theoremExplorerTab.theoremList} {previousPageClick} {nextPageClick}></TheoremList>
