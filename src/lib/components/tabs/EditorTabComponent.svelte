<script lang="ts" module>
  import EditorTabComponent from "$lib/components/tabs/EditorTabComponent.svelte";

  export class EditorTab extends Tab {
    component = EditorTabComponent;

    #filePath: string = $state("");
    #fileName: string = $derived.by(() => {
      let segments = this.#filePath.split("\\");
      return segments[segments.length - 1];
    });

    text: string = $state("");
    textChanged: boolean = $state(false);

    #monacoModel: Monaco.editor.ITextModel | null = null;

    constructor(filePath: string) {
      super();
      this.#filePath = filePath;
    }

    async loadData(): Promise<void> {
      this.text = await invoke("read_file", { relativePath: this.#filePath });
      this.#monacoModel = monaco.editor.createModel(this.text, "mmp");
    }

    unloadData(): void {
      this.text = "";
      this.textChanged = false;
      this.#monacoModel?.dispose();
      this.#monacoModel = null;
    }

    name(): string {
      return this.#fileName;
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof EditorTab && this.#filePath == tab.filePath;
    }

    showDot(): boolean {
      return this.textChanged;
    }

    changeEditorID(newID: string) {
      // this.#inProgressTheoremName = newID;
    }

    async deleteTheorem() {
      // await invoke("delete_in_progress_theorem", { name: this.#inProgressTheoremName });
      // tabManager.closeOpenTab();
      // nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
      // return;
    }

    async addToDatabase() {
      await invoke("add_to_database", { text: this.text });

      // let dataUnknown = await invoke("turn_into_theorem", { inProgressTheorem: this.#inProgressTheorem, positionName: placeAfter });
      // let theoremPath = dataUnknown as TheoremPath;
      // nameListData.removeInProgressTheoremName(this.#inProgressTheorem.name);
      // await explorerData.addTheoremName(theoremPath, this.#inProgressTheorem.name);
      // tabManager.changeTab(new TheoremTab(this.#inProgressTheorem.name));
    }

    get filePath() {
      return this.#filePath;
    }

    get fileName() {
      return this.#fileName;
    }

    get monacoModel() {
      return this.#monacoModel;
    }
  }
</script>

<script lang="ts">
  import type * as Monaco from "monaco-editor/esm/vs/editor/editor.api";
  import { invoke } from "@tauri-apps/api/core";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { Tab, tabManager } from "$lib/sharedState/tabManager.svelte";
  import { onDestroy, onMount } from "svelte";
  import monaco from "$lib/monaco/monaco";

  let { tab }: { tab: Tab } = $props();

  let editorTab = $derived.by(() => {
    if (tab instanceof EditorTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let editor: Monaco.editor.IStandaloneCodeEditor;
  // let monaco: typeof Monaco;
  let editorContainer: HTMLElement;

  onMount(async () => {
    editorContainer = document.getElementById("editor-area")!;
    editor = monaco.editor.create(editorContainer);
  });

  $effect(() => {
    editor?.setModel(editorTab.monacoModel);
  });

  onDestroy(() => {
    // monaco?.editor.getModels().forEach((model) => model.dispose());
    editor?.dispose();
  });

  // let lines = $derived.by(() => {
  //   let lines = 1;
  //   for (let char of editorTab.text) {
  //     if (char === "\n") {
  //       lines++;
  //     }
  //   }
  //   return lines;
  // });

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

  let saveText = () => {
    invoke("save_file", { relativePath: editorTab.filePath, content: editorTab.text });
    editorTab.textChanged = false;
  };

  let deleteTheorem = () => {
    editorTab.deleteTheorem();
  };

  let textChange = () => {
    tabManager.makeOpenTempTabPermanent();
    editorTab.textChanged = true;
  };

  let textareaKeyDown = async (e: KeyboardEvent) => {
    if (e.key === "u" && e.ctrlKey) {
      e.preventDefault();
      let textarea = document.getElementById("editorTextarea") as HTMLTextAreaElement;
      let resultText = (await invoke("unify", { text: textarea.value, cursorPos: textarea.selectionStart })) as string;
      if (resultText != editorTab.text) {
        editorTab.text = resultText;
        editorTab.textChanged = true;
      }
    }
  };

  let addToDatabase = () => {
    editorTab.addToDatabase();
  };

  let placeAfter = $state("");

  // $effect(() => {
  //   let textarea = document.getElementById("editorTextarea");
  //   if (textarea) {
  //     textarea.style.height = lines * 1.5 + "rem";
  //   }
  // });

  let belowTextareaClick = () => {
    let textarea = document.getElementById("editorTextarea");
    if (textarea && textarea instanceof HTMLTextAreaElement) {
      textarea.focus();
      textarea.setSelectionRange(-1, -1);
    }
  };
</script>

<!-- <div class="m-2">
  <div class="mb-2">
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={nameInput} onfocusout={onFocusOutName} onkeydown={onkeyDownName} disabled={nameDisabled} autocomplete="off" class="disabled:bg-gray-300" />
  </div>
  <RoundButton onclick={editName} disabled={!nameDisabled}>Edit name</RoundButton>
</div>
<div class="p-2 border-t border-gray-400">
  <RoundButton onclick={saveText} disabled={!editorTab.textChanged}>Save</RoundButton>
  <RoundButton onclick={deleteTheorem} warning>Delete</RoundButton>
</div>
<div class="p-2 border-t border-gray-400">
  <label for="placeAfter">Place after:</label>
  <input id="placeAfter" bind:value={placeAfter} autocomplete="off" />
  <RoundButton onclick={addToDatabase}>Add to database</RoundButton>
</div> -->
<div id="editor-area" class="h-full w-full">
  <!-- <div class="w-8 float-left text-right">
    {#each { length: lines } as _, i}
      <div>
        {i}
      </div>
    {/each}
  </div>
  <div class="ml-12">
    <textarea id="editorTextarea" bind:value={editorTab.text} oninput={textChange} onkeydown={textareaKeyDown} class="w-full resize-none h-96 text-nowrap overflow-x-hidden focus:outline-none" spellcheck="false"></textarea>
    <button class="w-full h-screen cursor-text" onclick={belowTextareaClick} aria-label="below-textrea"></button>
  </div> -->
</div>
