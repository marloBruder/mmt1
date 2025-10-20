<script lang="ts">
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerHeader from "./explorer/ExplorerHeader.svelte";
  import ExplorerButton from "./explorer/ExplorerButton.svelte";
  import { page } from "$app/stores";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import ExplorerTheoremButton from "./explorer/ExplorerTheoremButton.svelte";

  let filter = $state("");
  let quickSearchResults: string[] = $state([]);
  let more = $state(false);

  let newExplorerTabClick = () => {
    tabManager.openTab(new TheoremExplorerTab(0), true);
  };

  let quickSearchInput = async () => {
    [quickSearchResults, more] = await invoke("quick_search", { query: filter, onlyTen: true });
  };

  let loadAllQuickSearch = async () => {
    [quickSearchResults, more] = await invoke("quick_search", { query: filter, onlyTen: false });
    console.log(quickSearchResults);
  };
</script>

<div class="h-full w-full">
  <div class="p-2">
    <RoundButton onclick={newExplorerTabClick} additionalClasses="w-full">New Explorer Tab</RoundButton>
  </div>
  <div class="p-2">
    <div>Quick Search:</div>
    <div>
      <input bind:value={filter} oninput={quickSearchInput} class="custom-bg-input-color border border-gray-300 rounded w-full" />
    </div>
  </div>
  {#if filter === ""}
    <div class="pt-2">
      <ExplorerHeader header={explorerData.theoremListHeader} headerPath={{ path: [] }}></ExplorerHeader>
    </div>
  {:else}
    {#each quickSearchResults as theoremName}
      <ExplorerTheoremButton label={theoremName}></ExplorerTheoremButton>
    {/each}
    {#if more}
      <button onclick={loadAllQuickSearch}>Load all</button>
    {/if}
  {/if}
</div>
