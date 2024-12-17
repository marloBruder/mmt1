<script lang="ts">
  import inProgressTheoremData from "$lib/sharedState/mainData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { localID } = $props();

  let theorem = $derived(inProgressTheoremData.getTheoremByID(localID));

  let oldName: string = "";

  let nameDisabled: boolean = $state(true);

  let editName = () => {
    oldName = theorem ? theorem.name : "";
    nameDisabled = false;
  };

  let saveName = () => {
    if (theorem) {
      if (inProgressTheoremData.nameExists(theorem.id, oldName)) {
        return;
      }
      nameDisabled = true;
      if (oldName != theorem.name) {
        invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name });
      }
    }
  };

  let abortNameSave = () => {
    nameDisabled = true;
    if (theorem) {
      theorem.name = oldName;
    }
  };

  let textChanged: boolean = $state(false);

  let saveText = () => {
    if (theorem) {
      invoke("set_in_progress_theorem", { name: theorem.name, text: theorem.text });
      textChanged = false;
    }
  };

  let textChange = () => {
    textChanged = true;
  };
</script>

{#if theorem}
  <div class="m-2">
    <div class="mb-2">
      <label for="tabName">Theorem name:</label>
      <input id="tabName" type="text" bind:value={theorem.name} disabled={nameDisabled} class="disabled:bg-gray-300" />
    </div>
    <button onclick={editName} disabled={!nameDisabled} class="border border-black rounded px-1 disabled:bg-gray-300">Edit name</button>
    <button onclick={saveName} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Save name</button>
    <button onclick={abortNameSave} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Abort edit</button>
  </div>
  <div class="p-2 border-t border-gray-400">
    <button onclick={saveText} disabled={!textChanged} class="border border-black rounded px-1 disabled:bg-gray-300">Save</button>
  </div>
  <div>
    <textarea bind:value={theorem.text} oninput={textChange} class="w-full resize-none h-96"></textarea>
  </div>
{:else}
  <p>Opened editor tab with id "{localID}" has no data associated with it.</p>
{/if}
