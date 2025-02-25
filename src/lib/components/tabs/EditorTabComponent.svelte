<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { nameListData } from "$lib/sharedState/nameListData.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { EditorTab, tabManager, type Tab } from "$lib/sharedState/tabData.svelte";

  let { tab }: { tab: Tab } = $props();

  let editorTab = $derived.by(() => {
    if (tab instanceof EditorTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let nameInput: string = $state("");

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
    nameInput = editorTab.fileName;
    nameDisabled = false;
  };

  let saveName = async () => {
    // invoke("set_in_progress_theorem_name", { oldName, newName: theorem.name }).then(() => {
    //   nameDisabled = true;
    //   if (oldName != theorem.name) {
    //     nameListData.changeInProgressTheoremName(oldName, theorem.name);
    //     editorTab.changeEditorID(theorem.name);
    //   }
    // });
    nameDisabled = true;
  };

  let abortNameSave = () => {
    nameDisabled = true;
    nameInput = editorTab.fileName;
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
    invoke("save_file", { relativePath: editorTab.filePath, content: editorTab.text });
    textChanged = false;
  };

  let deleteTheorem = () => {
    editorTab.deleteTheorem();
  };

  let textChange = () => {
    tabManager.makeOpenTempTabPermanent();
    textChanged = true;
  };

  let turnIntoAxiom = () => {
    editorTab.convertToTheorem(placeAfter);
  };

  let placeAfter = $state("");

  $effect(() => {
    let textarea = document.getElementById("editorTextarea");
    if (textarea) {
      let lines = 1;
      for (let char of editorTab.text) {
        if (char === "\n") {
          lines++;
        }
      }
      console.log(lines);
      textarea.style.height = lines * 1.5 + "rem";
    }
  });
</script>

<div class="m-2">
  <div class="mb-2">
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={nameInput} onfocusout={onFocusOutName} onkeydown={onkeyDownName} disabled={nameDisabled} autocomplete="off" class="disabled:bg-gray-300" />
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
  <textarea id="editorTextarea" bind:value={editorTab.text} oninput={textChange} class="w-full resize-none h-96 text-nowrap overflow-x-hidden"></textarea>
</div>
