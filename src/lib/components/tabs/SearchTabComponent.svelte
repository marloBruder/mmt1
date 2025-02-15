<script lang="ts">
  import { SearchTab, type Tab } from "$lib/sharedState/tabData.svelte";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let searchTab = $derived.by(() => {
    if (tab instanceof SearchTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let previousPageClick = async () => {
    await searchTab.previousPage();
  };

  let nextPageClick = async () => {
    await searchTab.nextPage();
  };
</script>

<TheoremList theoremList={searchTab.searchResult} {previousPageClick} {nextPageClick}></TheoremList>
