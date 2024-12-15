<script lang="ts">
  import type { PageData } from "./$types";
  import editorTabs from "$lib/sharedState/mainData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { data }: { data: PageData } = $props();

  $effect(() => {
    editorTabs.openedTabID = data.tabID;
  });

  let tab = $derived(editorTabs.getTabByID(data.tabID));

  let oldName: string = $state("");

  let nameDisabled: boolean = $state(true);

  let editName = () => {
    oldName = tab ? tab.name : "";
    nameDisabled = false;
  };

  let saveName = () => {
    if (tab) {
      if (editorTabs.nameExists(tab.id, tab.name)) {
        return;
      }
      nameDisabled = true;
      if (oldName != tab.name) {
        invoke("set_in_progress_theorem_name", { oldName, newName: tab.name });
      }
    }
  };

  let abortNameSave = () => {
    nameDisabled = true;
    if (tab) {
      tab.name = oldName;
    }
  };

  let textChanged: boolean = $state(false);

  let saveText = () => {
    if (tab) {
      invoke("set_in_progress_theorem", { name: tab.name, text: tab.text });
      textChanged = false;
    }
  };

  let textChange = () => {
    textChanged = true;
  };
</script>

{#if tab}
  <div class="m-2">
    <div class="mb-2">
      <label for="tabName">Theorem name:</label>
      <input id="tabName" type="text" bind:value={tab.name} disabled={nameDisabled} class="disabled:bg-gray-300" />
    </div>
    <button onclick={editName} disabled={!nameDisabled} class="border border-black rounded px-1 disabled:bg-gray-300">Edit name</button>
    <button onclick={saveName} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Save name</button>
    <button onclick={abortNameSave} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Abort edit</button>
  </div>
  <div class="p-2 border-t border-gray-400">
    <button onclick={saveText} disabled={!textChanged} class="border border-black rounded px-1 disabled:bg-gray-300">Save</button>
  </div>
  <div>
    <textarea bind:value={tab.text} oninput={textChange} class="w-full resize-none h-96"></textarea>
  </div>
{:else}
  <p>Opened editor tab with id "{data.tabID}" has no data associated with it.</p>
{/if}
