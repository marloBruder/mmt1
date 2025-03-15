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
    tabManager.openTab(new TheoremExplorerTab(), true);
  };

  let quickSearchInput = async () => {
    [quickSearchResults, more] = await invoke("quick_search", { query: filter, onlyTen: true });
  };

  let loadAllQuickSearch = async () => {
    [quickSearchResults, more] = await invoke("quick_search", { query: filter, onlyTen: false });
    console.log(quickSearchResults);
  };
</script>

<div class="h-full">
  <div class="p-2">
    <RoundButton onclick={newExplorerTabClick}>New Explorer Tab</RoundButton>
  </div>
  <div class="p-2">
    Quick Search:
    <input bind:value={filter} oninput={quickSearchInput} class="border border-black rounded" />
  </div>
  <!-- <div class="pl-1 py-2">Explorer:</div> -->
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
  <!-- <div class="pl-1 py-2">Explorer:</div>
  <ul class="pl-2">
    {#each nameListData.theoremNames as name}
      {#if name.startsWith(filter)}
        <li class:bg-gray-300={name === theoremName}>
          <button class="pl-1" onclick={() => explorerClick(name)}>{name}</button>
        </li>
      {/if}
    {/each}
  </ul> -->
</div>
