<script lang="ts">
  import { goto } from "$app/navigation";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import VerticalDraggableSplit from "$lib/components/util/VerticalDraggableSplit.svelte";
  import monaco from "$lib/monaco/monaco";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";

  let onCancelClick = () => {
    goto("/main");
  };

  let editor: monaco.editor.IStandaloneDiffEditor | null = null;
  let oldMonacoModel: monaco.editor.ITextModel | null = null;
  let newMonacoModel: monaco.editor.ITextModel | null = null;

  onMount(async () => {
    const text = globalState.lastEditorContent;
    globalState.lastEditorContent = "";
    const res = (await invoke("add_to_database_preview", { text })) as [string, string] | null;

    if (res === null) {
      return;
    }

    const [oldFileContent, newFileContent] = res;

    const editorContainer = document.getElementById("editor-div")!;
    editor = monaco.editor.createDiffEditor(editorContainer, {
      automaticLayout: true,
      fixedOverflowWidgets: true,
      theme: "mmp-theme",
      minimap: { enabled: false },
      stickyScroll: { enabled: false },
      readOnly: true,
    });

    oldMonacoModel = monaco.editor.createModel(oldFileContent, "text");
    newMonacoModel = monaco.editor.createModel(newFileContent, "text");

    editor.setModel({ original: oldMonacoModel, modified: newMonacoModel });

    let changes = editor.getLineChanges();
    let changeLookups = 1;
    while (changes === null && changeLookups < 100) {
      await new Promise((r) => setTimeout(r, 100));
      changes = editor.getLineChanges();
      changeLookups += 1;
    }
    if (changes && changes.length > 0) {
      const firstChange = changes[0];
      const firstChangePos = firstChange.originalStartLineNumber;
      editor.revealLineInCenter(firstChangePos);
    }
  });

  onDestroy(() => {
    oldMonacoModel?.dispose();
    newMonacoModel?.dispose();
    editor?.dispose();
  });
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <HorizontalSplit>
    {#snippet first()}
      <div class="w-full text-left py-2">
        <button class="pl-4" onclick={onCancelClick}>{"< Cancel"}</button>
      </div>
      <div class="text-center text-3xl pt-2 pb-10">Add to Database</div>
    {/snippet}
    {#snippet second()}
      <div class="h-full">
        <VerticalDraggableSplit startPercent={0.8}>
          {#snippet first()}
            <div class="px-2 w-full h-full">
              <div id="editor-div" class="w-full h-full"></div>
            </div>
          {/snippet}
          {#snippet second()}
            <div class="ml-2 h-full w-full border-t border-l rounded-tl-lg">
              <ScrollableContainer>
                <div></div>
              </ScrollableContainer>
            </div>
          {/snippet}
        </VerticalDraggableSplit>
      </div>
    {/snippet}
  </HorizontalSplit>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
