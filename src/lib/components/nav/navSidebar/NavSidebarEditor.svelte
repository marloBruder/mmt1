<script lang="ts">
  import { page } from "$app/stores";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import { EditorTab, tabManager } from "$lib/sharedState/tabData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let theoremName: string | null = $derived.by(() => {
    let segments = $page.url.pathname.split("/");
    if (segments.length == 4 && segments[1] == "main" && segments[2] == "editor") {
      return segments[3];
    }
    return null;
  });

  let theoremClick = (inProgressTheoremName: string) => {
    tabManager.openTab(new EditorTab(inProgressTheoremName));
  };

  let theoremDblClick = (inProgressTheoremName: string) => {
    tabManager.makeSameTempTabPermanent(new EditorTab(inProgressTheoremName));
  };

  let newTabName = $state("");

  let addTheoremClick = async () => {
    invoke("add_in_progress_theorem", { name: newTabName, text: "" }).then(async () => {
      let name = newTabName;
      nameListData.addInProgressTheoremName(name);
      newTabName = "";
      await tabManager.openTab(new EditorTab(name), true);
    });
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
  <div class="pl-1">In Progress theorems:</div>
  <ul class="pl-2 pt-1">
    {#each nameListData.inProgressTheoremNames as name}
      <li class:bg-gray-300={theoremName == name}>
        <button class="pl-1" onclick={() => theoremClick(name)} ondblclick={() => theoremDblClick(name)}>{name}</button>
      </li>
    {/each}
  </ul>
</div>
