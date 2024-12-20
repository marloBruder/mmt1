<script lang="ts">
  import { inProgressTheoremData } from "$lib/sharedState/metamathData/inProgressTheoremData.svelte";
  import { tabManager, EditorTab } from "$lib/sharedState/tabData.svelte";

  let theoremClick = (id: number) => {
    return () => {
      tabManager.addTabAndOpen(new EditorTab(id));
    };
  };
</script>

<div>
  <div class="py-4 flex flex-col items-center">
    <button class="border border-black" onclick={() => inProgressTheoremData.addDefaultTheorem()}>Add new theorem</button>
  </div>
  <div>In Progress theorems:</div>
  {#each inProgressTheoremData.theorems as theorem}
    <div>
      <button onclick={theoremClick(theorem.id)}>{theorem.name}</button>
    </div>
  {/each}
</div>
