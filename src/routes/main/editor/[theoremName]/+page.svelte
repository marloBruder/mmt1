<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import type { PageData } from "./$types";

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

  $effect(() => {
    if (tab.inProgressTheoremName === "") {
      oldName = "";
      nameDisabled = false;
    }
  });

  let editName = () => {
    oldName = theorem.name;
    nameDisabled = false;
  };

  let saveName = async () => {
    if (theorem.name === "" || (oldName != theorem.name && nameListData.nameExists(theorem.name))) {
      throw Error("Invalid Name");
    }
    nameDisabled = true;
    if (oldName != theorem.name) {
      if (oldName === "") {
        invoke("add_in_progress_theorem", { name: theorem.name, text: "" });
        nameListData.addInProgressTheoremName(theorem.name);
      } else {
        invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name });
        nameListData.changeInProgressTheoremName(oldName, theorem.name);
      }
      tab.changeID(theorem.name);
    }
  };

  let abortNameSave = () => {
    if (tab.inProgressTheoremName === "") {
      // tabManager.closeCurrentTab();
    } else {
      nameDisabled = true;
      theorem.name = oldName;
    }
  };

  let unfocusName = async () => {
    try {
      await saveName();
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
    tab.convertToTheorem();
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
