<script lang="ts" module>
  import EditorTabComponent from "$lib/components/tabs/EditorTabComponent.svelte";

  let editor: Monaco.editor.IStandaloneCodeEditor;

  export class EditorTab extends Tab {
    component = EditorTabComponent;

    #filePath: string = $state("");
    #fileName: string = $derived.by(() => {
      let segments = this.#filePath.split("\\");
      return segments[segments.length - 1];
    });

    textChanged: boolean = $state(false);

    #monacoModel: Monaco.editor.ITextModel | null = null;
    #monacoScrollTop: number = 0;
    #monacoScrollLeft: number = 0;

    constructor(filePath: string) {
      super();
      this.#filePath = filePath;
    }

    async loadData(): Promise<void> {
      let text = (await invoke("read_file", { relativePath: this.#filePath })) as string;
      this.#monacoModel = monaco.editor.createModel(text, "mmp");
      this.#monacoModel.onDidChangeContent(() => {
        this.textChanged = true;
        tabManager.makeOpenTempTabPermanent();
      });
    }

    unloadData(): void {
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

    async saveFile(): Promise<void> {
      await invoke("save_file", { relativePath: this.#filePath, content: this.#monacoModel!.getValue() });
      this.textChanged = false;
    }

    saveFileDisabled(): boolean {
      return !this.textChanged;
    }

    async unify(): Promise<void> {
      let resultText = (await invoke("unify", { text: this.monacoModel!.getValue() })) as string;
      if (resultText != this.monacoModel!.getValue()) {
        // this.monacoModel!.setValue(resultText);
        editor.executeEdits("unifier", [{ /*identifier: "delete" as any,*/ range: new monaco.Range(1, 1, 10000, 1), text: resultText, forceMoveMarkers: true }]);
        // editor.executeEdits("unifier", [{ /*identifier: "insert" as any,*/ range: new monaco.Range(1, 1, 1, 1), text: resultText, forceMoveMarkers: true }]);
      }
    }

    unifyDisabled(): boolean {
      return false;
    }

    async addToDatabase() {
      await invoke("add_to_database", { text: this.monacoModel!.getValue() });

      // let dataUnknown = await invoke("turn_into_theorem", { inProgressTheorem: this.#inProgressTheorem, positionName: placeAfter });
      // let theoremPath = dataUnknown as TheoremPath;
      // nameListData.removeInProgressTheoremName(this.#inProgressTheorem.name);
      // await explorerData.addTheoremName(theoremPath, this.#inProgressTheorem.name);
      // tabManager.changeTab(new TheoremTab(this.#inProgressTheorem.name));
    }

    addToDatabaseDisabled(): boolean {
      return false;
    }

    setMonacoScrollInternal(scrollTop: number, scrollLeft: number) {
      this.#monacoScrollTop = scrollTop;
      this.#monacoScrollLeft = scrollLeft;
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
    get monacoScrollTop() {
      return this.#monacoScrollTop;
    }
    get monacoScrollLeft() {
      return this.#monacoScrollLeft;
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

  // let monaco: typeof Monaco;
  let editorContainer: HTMLElement;

  onMount(async () => {
    editorContainer = document.getElementById("editor-area")!;
    editor = monaco.editor.create(editorContainer, { automaticLayout: true, theme: "mmp-theme" });

    editor.addAction({
      id: "unify-action",
      label: "Unify",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyU],
      run: async () => {
        await editorTab.unify();
      },
    });

    editor.addAction({
      id: "save-action",
      label: "Save file",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
      run: async () => {
        await editorTab.saveFile();
      },
    });

    editor.onDidScrollChange((e) => {
      editorTab.setMonacoScrollInternal(e.scrollTop, e.scrollLeft);
    });
  });

  $effect(() => {
    editor?.setModel(editorTab.monacoModel);
    editor?.setScrollTop(editorTab.monacoScrollTop);
    editor?.setScrollLeft(editorTab.monacoScrollLeft);
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

  let textChange = () => {
    tabManager.makeOpenTempTabPermanent();
    editorTab.textChanged = true;
  };

  let textareaKeyDown = async (e: KeyboardEvent) => {
    if (e.key === "u" && e.ctrlKey) {
      e.preventDefault();
      let textarea = document.getElementById("editorTextarea") as HTMLTextAreaElement;
      let resultText = (await invoke("unify", { text: textarea.value, cursorPos: textarea.selectionStart })) as string;
      if (resultText != editorTab.monacoModel?.getValue()) {
        editorTab.monacoModel?.setValue(resultText);
        // editorTab.textChanged = true;
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
