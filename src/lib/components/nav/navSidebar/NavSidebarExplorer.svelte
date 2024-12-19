<script lang="ts">
  import { theoremData } from "$lib/sharedState/metamathData/theoremData.svelte";
</script>

<div>
  {#each theoremData.theorems as theorem}
    <div>
      <button>{theorem.name}</button>
    </div>
  {/each}
</div>
