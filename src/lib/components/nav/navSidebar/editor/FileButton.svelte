<script lang="ts">
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";

  let { fileName }: { fileName: string } = $props();

  let explorerClick = (name: string) => {
    // tabManager.openTab(new TheoremTab(name));
  };

  let explorerDblClick = (name: string) => {
    // tabManager.makeSameTempTabPermanent(new TheoremTab(name));
  };
</script>

<div>
  <button class="w-full text-left pl-2 hover:bg-gray-200" onclick={() => explorerClick(fileName)} ondblclick={() => explorerDblClick(fileName)}>{fileName}</button>
</div>
