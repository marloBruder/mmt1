<script lang="ts">
  import { goto } from "$app/navigation";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";

  let theoremClick = (inProgressTheoremName: string) => {
    goto("/main/editor/" + inProgressTheoremName);
  };
  let addTheoremClick = () => {
    // tabManager.addTabAndOpen(new EditorTab(""));
  };
</script>

<div>
  <div class="py-4 flex flex-col items-center">
    <button class="border border-black" onclick={addTheoremClick}>Add new theorem</button>
  </div>
  <div>In Progress theorems:</div>
  {#each nameListData.inProgressTheoremNames as name}
    <div>
      <button onclick={() => theoremClick(name)}>{name}</button>
    </div>
  {/each}
</div>
