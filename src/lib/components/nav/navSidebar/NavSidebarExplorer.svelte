<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";

  let filter = $state("");

  let theoremName: string | null = $derived.by(() => {
    let segments = $page.url.pathname.split("/");
    if (segments.length == 4 && segments[1] == "main" && segments[2] == "theorem") {
      return segments[3];
    }
    return null;
  });

  let explorerClick = (name: string) => {
    goto("/main/theorem/" + name);
  };
</script>

<div>
  <div class="p-2">
    Quick Search:
    <input bind:value={filter} class="border border-black rounded" />
  </div>
  <div class="pl-1 py-2">Explorer:</div>
  <ul class="pl-2">
    {#each nameListData.theoremNames as name}
      {#if name.startsWith(filter)}
        <li class:bg-gray-300={name === theoremName}>
          <button class="pl-1" onclick={() => explorerClick(name)}>{name}</button>
        </li>
      {/if}
    {/each}
  </ul>
</div>
