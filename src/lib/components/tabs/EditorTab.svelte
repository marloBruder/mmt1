<script lang="ts">
  import { inProgressTheoremData } from "$lib/sharedState/mainData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  import { tabManager } from "$lib/sharedState/tabData.svelte";

  let { localID } = $props();

  let theorem = $derived.by(() => {
    let theoremOrNull = inProgressTheoremData.getTheoremByID(localID);
    if (theoremOrNull === null) {
      // This should never happen, if the tabManager editor-tabs always had a correct id
      tabManager.openEmptyTab();
      return { id: 0, name: "", text: "" };
    }
    return theoremOrNull;
  });

  let oldName: string = "";

  let nameDisabled: boolean = $state(true);

  let editName = () => {
    oldName = theorem.name;
    nameDisabled = false;
  };

  let saveName = () => {
    if (theorem.name === "" || inProgressTheoremData.nameExists(theorem.id, theorem.name)) {
      throw Error("Invalid Name");
    }
    nameDisabled = true;
    if (oldName != theorem.name) {
      invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name });
    }
  };

  let abortNameSave = () => {
    nameDisabled = true;
    theorem.name = oldName;
  };

  let unfocusName = () => {
    try {
      saveName();
    } catch (error) {
      abortNameSave();
    }
  };

  let keyDownName = (event: KeyboardEvent) => {
    if (event.key == "Enter") {
      try {
        saveName();
      } catch (error) {}
    } else if (event.key == "Escape") {
      abortNameSave();
    }
  };

  let textChanged: boolean = $state(false);

  let saveText = () => {
    invoke("set_in_progress_theorem", { name: theorem.name, text: theorem.text });
    textChanged = false;
  };

  let deleteTheorem = () => {
    inProgressTheoremData.deleteTheorem(theorem.id);
  };

  let textChange = () => {
    textChanged = true;
  };
</script>

<div class="m-2">
  <div class="mb-2">
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={theorem.name} onfocusout={unfocusName} onkeydown={keyDownName} disabled={nameDisabled} autocomplete="off" class="disabled:bg-gray-300" />
  </div>
  <button onclick={editName} disabled={!nameDisabled} class="border border-black rounded px-1 disabled:bg-gray-300">Edit name</button>
</div>
<div class="p-2 border-t border-gray-400">
  <button onclick={saveText} disabled={!textChanged} class="border border-black rounded px-1 disabled:bg-gray-300">Save</button>
  <button onclick={deleteTheorem} class="border border-black rounded px-1 bg-red-500">Delete</button>
</div>
<div>
  <textarea bind:value={theorem.text} oninput={textChange} class="w-full resize-none h-96"></textarea>
</div>
