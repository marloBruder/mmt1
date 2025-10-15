<script lang="ts">
  import { goto } from "$app/navigation";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import SelectDropdown, { type SelectDropdownOption } from "$lib/components/util/SelectDropdown.svelte";
  import VerticalDraggableSplit from "$lib/components/util/VerticalDraggableSplit.svelte";
  import monaco from "$lib/monaco/monaco";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import { settingsData } from "$lib/sharedState/settingsData.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";

  let onCancelClick = () => {
    goto("/main");
  };

  let editor: monaco.editor.IStandaloneDiffEditor | null = null;
  let oldMonacoModel: monaco.editor.ITextModel | null = null;
  let newMonacoModel: monaco.editor.ITextModel | null = null;

  let loading = $state(true);

  onMount(async () => {
    const text = globalState.lastEditorContent;
    const res = (await invoke("add_to_database_preview", { text, overrideProofFormat: null })) as [string, string] | null;

    if (res === null) {
      loading = false;
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
    scrollToChange(changes);
    loading = false;
  });

  let scrollToChange = (changes?: monaco.editor.ILineChange[] | null) => {
    let c = changes !== undefined ? changes : editor?.getLineChanges();

    if (c && c.length > 0) {
      const firstChange = c[0];
      const firstChangePos = firstChange.originalStartLineNumber;
      editor?.revealLineInCenter(firstChangePos);
    }
  };

  onDestroy(() => {
    oldMonacoModel?.dispose();
    newMonacoModel?.dispose();
    editor?.dispose();
    globalState.lastEditorContent = "";
  });

  let proofFormatOption: "compressed" | "uncompressed" = $state(settingsData.settings.proofFormat);

  let proofFormatOptions: SelectDropdownOption[] = [
    { label: "Uncompressed", value: "uncompressed" },
    { label: "Compressed", value: "compressed" },
  ];

  $effect(() => {
    loading = true;
    const text = globalState.lastEditorContent;
    invoke("add_to_database_preview", { text, overrideProofFormat: proofFormatOption }).then(async (resUnkown) => {
      const res = resUnkown as [string, string] | null;

      if (res === null) {
        loading = false;
        return;
      }

      const [oldFileContent, newFileContent] = res;

      oldMonacoModel?.setValue(oldFileContent);
      newMonacoModel?.setValue(newFileContent);

      let changes = editor?.getLineChanges();
      let changeLookups = 1;
      while (changes === null && changeLookups < 100) {
        await new Promise((r) => setTimeout(r, 100));
        changes = editor?.getLineChanges();
        changeLookups += 1;
      }
      scrollToChange(changes);
      loading = false;
    });
  });
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <HorizontalSplit>
    {#snippet first()}
      <div class="w-full text-left py-2">
        <button class="pl-4" onclick={onCancelClick}>{"< Cancel"}</button>
      </div>
      <div class="text-center pb-10"><h1 class="text-3xl">Add to Database</h1></div>
    {/snippet}
    {#snippet second()}
      <div class="h-full overflow-hidden">
        <VerticalDraggableSplit startPercent={0.8}>
          {#snippet first()}
            <div class="w-full h-full">
              {#if loading}
                <div class="w-full h-full flex justify-center items-center">
                  <div>Loading...</div>
                </div>
              {/if}
              <div class={"px-2 w-full h-full " + (loading ? " invisible " : "")}>
                <div id="editor-div" class="w-full h-full"></div>
              </div>
            </div>
          {/snippet}
          {#snippet second()}
            <div class="ml-2 p-2 h-full border-t border-l rounded-tl-lg">
              <ScrollableContainer>
                <div class="h-full w-full">
                  <h2 class="text-xl">Options</h2>
                  <div class="py-2">
                    <hr />
                  </div>
                  <div class="pt-2">
                    Proof format:
                    <SelectDropdown bind:value={proofFormatOption} options={proofFormatOptions}></SelectDropdown>
                  </div>
                  <div class="pt-6">
                    <RoundButton onclick={() => scrollToChange()} additionalClasses="w-full" disabled={loading}>Scroll to change</RoundButton>
                  </div>
                  <div class="py-2">
                    <hr />
                  </div>
                </div>
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
