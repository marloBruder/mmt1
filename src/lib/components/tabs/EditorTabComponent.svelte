<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import type { PageData } from "../../../routes/main/editor/[theoremName]/$types";
  import { goto } from "$app/navigation";

  let { data }: { data: PageData } = $props();

  let tab = $derived(data.tab);
  let theorem = $derived(tab.inProgressTheorem);

  let oldName: string = "";

  let nameDisabled: boolean = $state(true);

  $effect(() => {
    if (!nameDisabled) {
      let input = document.getElementById("tabName");
      if (input) {
        input.focus();
      }
    }
  });

  let editName = () => {
    oldName = theorem.name;
    nameDisabled = false;
  };

  let saveName = async () => {
    invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name }).then(() => {
      nameDisabled = true;
      if (oldName != theorem.name) {
        nameListData.changeInProgressTheoremName(oldName, theorem.name);
        tab.changeEditorID(theorem.name);
        goto("/main/editor/" + theorem.name);
      }
    });

    // if (oldName != theorem.name && !nameListData.validNewName(theorem.name)) {
    //   throw Error("Invalid Name");
    // }
    // nameDisabled = true;
    // if (oldName != theorem.name) {
    //   invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name });
    //   nameListData.changeInProgressTheoremName(oldName, theorem.name);
    //   tab.changeEditorID(theorem.name);
    //   goto("/main/editor/" + theorem.name);
    // }
  };

  let abortNameSave = () => {
    nameDisabled = true;
    theorem.name = oldName;
  };

  let onFocusOutName = async () => {
    try {
      await saveName();
    } catch (error) {
      abortNameSave();
    }
  };

  let onkeyDownName = (event: KeyboardEvent) => {
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
    invoke("set_in_progress_theorem", { name: tab.inProgressTheoremName, text: theorem.text });
    textChanged = false;
  };

  let deleteTheorem = () => {
    tab.deleteTheorem();
  };

  let textChange = () => {
    textChanged = true;
  };

  let turnIntoAxiom = () => {
    tab.convertToTheorem(placeAfter);
  };

  let placeAfter = $state("");
</script>

<div class="m-2">
  <div class="mb-2">
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={theorem.name} onfocusout={onFocusOutName} onkeydown={onkeyDownName} disabled={nameDisabled} autocomplete="off" class="disabled:bg-gray-300" />
  </div>
  <RoundButton onclick={editName} disabled={!nameDisabled}>Edit name</RoundButton>
</div>
<div class="p-2 border-t border-gray-400">
  <RoundButton onclick={saveText} disabled={!textChanged}>Save</RoundButton>
  <RoundButton onclick={deleteTheorem} warning>Delete</RoundButton>
</div>
<div class="p-2 border-t border-gray-400">
  <label for="placeAfter">Place after:</label>
  <input id="placeAfter" bind:value={placeAfter} autocomplete="off" />
  <RoundButton onclick={turnIntoAxiom}>Turn into theorem</RoundButton>
</div>
<div>
  <textarea bind:value={theorem.text} oninput={textChange} class="w-full resize-none h-96"></textarea>
</div>
