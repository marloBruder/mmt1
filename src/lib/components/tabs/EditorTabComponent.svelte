<script lang="ts" module>
  import EditorTabComponent from "$lib/components/tabs/EditorTabComponent.svelte";

  let editor: Monaco.editor.IStandaloneCodeEditor;

  export class EditorTab extends Tab {
    component = EditorTabComponent;
    splitComponent = EditorTabSplitViewComponent;

    #filePath: string = $state("");
    #fileName: string = $derived.by(() => {
      const segments = this.#filePath.split("\\");
      return segments[segments.length - 1];
    });
    #isMmpFile: boolean = $derived(this.#fileName.endsWith(".mmp"));

    textChanged: boolean = $state(false);

    splitViewPageData: DatabaseElementPageData | null = $state(null);

    #monacoModel: Monaco.editor.ITextModel | null = null;
    #monacoScrollTop: number = 0;
    #monacoScrollLeft: number = 0;

    constructor(filePath: string) {
      super();
      this.#filePath = filePath;
    }

    async loadData(): Promise<void> {
      const text = (await invoke("open_file", { relativePath: this.#filePath })) as string;
      this.#monacoModel = monaco.editor.createModel(text, this.#isMmpFile ? "mmp" : "text");
      this.#monacoModel.onDidChangeContent(async () => {
        this.textChanged = true;
        tabManager.makeOpenTempTabPermanent();
        await this.onMonacoChange();
      });
    }

    unloadData(): void {
      this.textChanged = false;
      this.#monacoModel?.dispose();
      this.#monacoModel = null;
      invoke("close_file", { relativePath: this.#filePath });
    }

    name(): string {
      return this.#fileName;
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof EditorTab && this.#filePath == tab.filePath;
    }

    async onTabOpen(): Promise<void> {
      await this.onMonacoChange();
    }

    showUnsavedChanges(): boolean {
      return this.textChanged;
    }

    async saveFile(): Promise<void> {
      if (settingsData.settings.formatOnSave) {
        await this.format();
      }

      await invoke("save_file", { relativePath: this.#filePath, content: this.#monacoModel!.getValue() });
      this.textChanged = false;
    }

    saveFileDisabled(): boolean {
      return !this.textChanged;
    }

    async unify(): Promise<void> {
      if (!this.#isMmpFile) {
        return;
      }

      const resultText = (await invoke("unify", { text: this.monacoModel!.getValue() })) as string | null;

      if (resultText !== null) {
        changeEditorTextMaintainCursor(resultText);
      }
    }

    unifyDisabled(): boolean {
      return !this.#isMmpFile;
    }

    async addToDatabase() {
      if (!this.#isMmpFile) {
        return;
      }

      globalState.lastEditorContent = this.#monacoModel!.getValue();
      goto("/main/addToDatabase");
    }

    addToDatabaseDisabled(): boolean {
      return !this.#isMmpFile;
    }

    formatDisabled(): boolean {
      return !this.#isMmpFile;
    }

    async format() {
      if (!this.#isMmpFile) {
        return;
      }

      const resultText = (await invoke("format", { text: this.#monacoModel!.getValue() })) as string | null;
      if (resultText !== null) {
        changeEditorTextMaintainCursor(resultText);
      }
    }

    renumberDisabled(): boolean {
      return !this.#isMmpFile;
    }

    async renumber() {
      if (!this.#isMmpFile) {
        return;
      }

      const resultText = (await invoke("renumber", { text: this.#monacoModel!.getValue() })) as string | null;
      if (resultText !== null) {
        changeEditorTextMaintainCursor(resultText);
      }
    }

    setMonacoScrollInternal(scrollTop: number, scrollLeft: number) {
      this.#monacoScrollTop = scrollTop;
      this.#monacoScrollLeft = scrollLeft;
    }

    async onMonacoChange() {
      if (!this.#isMmpFile) {
        return;
      }

      monaco.editor.removeAllMarkers("on_edit");
      invoke("on_edit", { text: this.monacoModel!.getValue() }).then(async (onEditDataUnkown) => {
        interface OnEditData {
          pageData: DatabaseElementPageData | null;
          errors: DetailedError[];
        }

        const onEditData = onEditDataUnkown as OnEditData;

        this.splitViewPageData = onEditData.pageData;

        if (tabManager.splitTabState === "externalWindow") {
          await emit("split-view-page-data-transfer", tabManager.getOpenTab()?.splitViewPageData);
        }

        const markers: Monaco.editor.IMarkerData[] = onEditData.errors.map((detailedError) => {
          return {
            severity: getErrorSeverity(detailedError.errorType),
            startLineNumber: detailedError.startLineNumber,
            startColumn: detailedError.startColumn,
            endLineNumber: detailedError.endLineNumber,
            endColumn: detailedError.endColumn,
            message: getErrorMessage(detailedError.errorType),
          };
        });

        monaco.editor.setModelMarkers(this.#monacoModel!, "on_edit", markers);
      });
    }

    get filePath() {
      return this.#filePath;
    }

    set filePath(newFilePath: string) {
      this.#filePath = newFilePath;
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

  function changeEditorTextMaintainCursor(newtext: string) {
    const selection = editor.getSelection();
    const cursorPosition = selection?.getPosition();

    editor.executeEdits("unifier", [{ range: new monaco.Range(1, 1, 1000000, 1), text: newtext, forceMoveMarkers: true }]);
    if (cursorPosition !== undefined) {
      editor.setPosition(cursorPosition);
    }
  }
</script>

<script lang="ts">
  import type * as Monaco from "monaco-editor/esm/vs/editor/editor.api";
  import { invoke } from "@tauri-apps/api/core";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { onDestroy, onMount, type Component } from "svelte";
  import monaco from "$lib/monaco/monaco";
  import type { DatabaseElementPageData, DetailedError } from "$lib/sharedState/model.svelte";
  import { getErrorMessage, getErrorSeverity } from "../util/errorMessages.svelte";
  import EditorTabSplitViewComponent from "./EditorTabSplitViewComponent.svelte";
  import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import { Tab } from "$lib/sharedState/tab.svelte";
  import { settingsData } from "$lib/sharedState/settingsData.svelte";

  let { tab }: { tab: Tab } = $props();

  let editorTab = $derived.by(() => {
    if (tab instanceof EditorTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  // let monaco: typeof Monaco;
  let editorContainer: HTMLElement;

  let unlistenFns: UnlistenFn[] = $state([]);

  onMount(async () => {
    editorContainer = document.getElementById("editor-area")!;
    editor = monaco.editor.create(editorContainer, {
      automaticLayout: true,
      fixedOverflowWidgets: true,
      theme: "mmp-theme",
      minimap: { enabled: false },
      stickyScroll: { enabled: false },
      wordBasedSuggestions: "off",
    });

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

    editor.addAction({
      id: "format-action",
      label: "Format",
      keybindings: [monaco.KeyMod.Shift | monaco.KeyMod.Alt | monaco.KeyCode.KeyF],
      run: async () => {
        await editorTab.format();
      },
    });

    editor.addAction({
      id: "renumber-action",
      label: "Renumber",
      keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR],
      run: async () => {
        await editorTab.renumber();
      },
    });

    editor.onDidScrollChange((e) => {
      editorTab.setMonacoScrollInternal(e.scrollTop, e.scrollLeft);
    });

    unlistenFns.push(
      await listen("request-first-split-view-transfer", () => {
        emit("split-view-page-data-transfer", tabManager.getOpenTab()?.splitViewPageData);
      })
    );
    unlistenFns.push(
      await listen("mm-db-opened", () => {
        editorTab.onMonacoChange();
      })
    );
    unlistenFns.push(
      await listen("grammar-calculations-performed", () => {
        editorTab.onMonacoChange();
      })
    );
  });

  $effect(() => {
    editor?.setModel(editorTab.monacoModel);
    editor?.setScrollTop(editorTab.monacoScrollTop);
    editor?.setScrollLeft(editorTab.monacoScrollLeft);
  });

  onDestroy(() => {
    // monaco?.editor.getModels().forEach((model) => model.dispose());
    editor?.dispose();

    for (let unlisten of unlistenFns) {
      unlisten();
    }
  });
</script>

<div class="h-full w-full">
  <div id="editor-area" class="h-full w-full"></div>
</div>
