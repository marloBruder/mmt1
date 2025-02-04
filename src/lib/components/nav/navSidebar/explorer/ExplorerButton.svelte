<script lang="ts">
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";

  let { theoremName, openTheoremName }: { theoremName: string; openTheoremName: string | null } = $props();

  let explorerClick = (name: string) => {
    tabManager.changeTab(new TheoremTab(name));
  };
</script>

<div>
  <button class={"w-full text-left pl-2 " + (theoremName === openTheoremName ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={() => explorerClick(theoremName)}>{theoremName}</button>
</div>
