<script lang="ts">
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import ExplorerHeader from "./explorer/ExplorerHeader.svelte";

  let filter = $state("");
</script>

<div>
  <div class="p-2">
    Quick Search:
    <input bind:value={filter} class="border border-black rounded" />
  </div>
  <!-- <div class="pl-1 py-2">Explorer:</div> -->
  <div>
    <ExplorerHeader header={nameListData.topHeader} location={[]}></ExplorerHeader>
  </div>
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
