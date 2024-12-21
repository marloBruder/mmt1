<script lang="ts">
  import { inProgressTheoremData } from "$lib/sharedState/metamathData/inProgressTheoremData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  import { tabManager } from "$lib/sharedState/tabData.svelte";
  import { theoremData } from "$lib/sharedState/metamathData/theoremData.svelte";
  import RoundButton from "../util/RoundButton.svelte";

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

  $effect(() => {
    if (!nameDisabled) {
      let input = document.getElementById("tabName");
      if (input) {
        input.focus();
      }
    }
  });

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

  let turnIntoAxiom = () => {
    theoremData.convertToTheorem(theorem.id);
  };
</script>

<div class="m-2">
  <div class="mb-2">
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={theorem.name} onfocusout={unfocusName} onkeydown={keyDownName} disabled={nameDisabled} autocomplete="off" class="disabled:bg-gray-300" />
  </div>
  <RoundButton onclick={editName} disabled={!nameDisabled}>Edit name</RoundButton>
</div>
<div class="p-2 border-t border-gray-400">
  <RoundButton onclick={saveText} disabled={!textChanged}>Save</RoundButton>
  <RoundButton onclick={deleteTheorem} warning>Delete</RoundButton>
  <RoundButton onclick={turnIntoAxiom}>Turn into axiom</RoundButton>
</div>
<div>
  <textarea bind:value={theorem.text} oninput={textChange} class="w-full resize-none h-96"></textarea>
</div>
