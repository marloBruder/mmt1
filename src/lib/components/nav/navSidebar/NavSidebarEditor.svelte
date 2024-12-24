<script lang="ts">
  import { goto } from "$app/navigation";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let theoremClick = (inProgressTheoremName: string) => {
    goto("/main/editor/" + inProgressTheoremName);
  };

  let newTabName = $state("");

  let addTheoremClick = async () => {
    if (nameListData.validNewName(newTabName)) {
      let name = newTabName;
      await invoke("add_in_progress_theorem", { name, text: "" });
      nameListData.addInProgressTheoremName(name);
      newTabName = "";
      goto("/main/editor/" + name);
    }
  };

  let onKeyDownName = (event: KeyboardEvent) => {
    if (event.key == "Enter") {
      addTheoremClick();
    }
  };
</script>

<div>
  <div class="py-4 flex flex-col items-center">
    <div class="pb-1">
      <label for="newTabName">Name:</label>
      <input id="newTabName" bind:value={newTabName} onkeydown={onKeyDownName} autocomplete="off" class="border rounded-sm border-black" />
    </div>
    <button class="border border-black" onclick={addTheoremClick}>Add new theorem</button>
  </div>
  <div>In Progress theorems:</div>
  {#each nameListData.inProgressTheoremNames as name}
    <div>
      <button onclick={() => theoremClick(name)}>{name}</button>
    </div>
  {/each}
</div>
