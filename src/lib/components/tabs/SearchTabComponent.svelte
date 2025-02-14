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
</script>

<TheoremList theoremList={searchTab.searchResult}></TheoremList>
