<script lang="ts">
  import { goto } from "$app/navigation";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";

  let explorerClick = (name: string) => {
    goto("/main/theorem/" + name);
  };
</script>

<div>
  <div class="py-2">Explorer:</div>
  {#each nameListData.theoremNames as name}
    <div>
      <button onclick={() => explorerClick(name)}>{name}</button>
    </div>
  {/each}
</div>
