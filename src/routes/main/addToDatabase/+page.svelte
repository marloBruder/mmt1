<script lang="ts">
  import { goto } from "$app/navigation";
  import { getAddToDatabaseErrorMessage } from "$lib/components/util/errorMessages.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import SelectDropdown, { type SelectDropdownOption } from "$lib/components/util/SelectDropdown.svelte";
  import VerticalDraggableSplit from "$lib/components/util/VerticalDraggableSplit.svelte";
  import monaco from "$lib/monaco/monaco";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import type { AddToDatabaseResult } from "$lib/sharedState/model.svelte";
  import { settingsData } from "$lib/sharedState/settingsData.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let canceled = $state(false);

  let addingInvalidHtml = $state(false);

  let onCancelClick = () => {
    canceled = true;
    goto("/main");
  };

  let editor: monaco.editor.IStandaloneDiffEditor | null = null;
  let oldMonacoModel: monaco.editor.ITextModel | null = null;
  let newMonacoModel: monaco.editor.ITextModel | null = null;

  let loading = $state(true);
  let previewError: string | null = $state(null);
  let addToDatabaseError: string | null = $state(null);

  interface AddToDatabasePreviewData {
    oldFileContent: string;
    newFileContent: string;
    invalidHtml: boolean;
  }

  onMount(async () => {
    const text = globalState.lastEditorContent;
    await invoke("add_to_database_preview", { text, overrideProofFormat: null })
      .then(async (dataUnknown) => {
        const data = dataUnknown as AddToDatabasePreviewData | null;

        if (data === null || canceled) {
          loading = false;
          return;
        }

        addingInvalidHtml = data.invalidHtml;

        const editorContainer = document.getElementById("editor-div")!;
        editor = monaco.editor.createDiffEditor(editorContainer, {
          automaticLayout: true,
          fixedOverflowWidgets: true,
          theme: "mmp-theme",
          minimap: { enabled: false },
          stickyScroll: { enabled: false },
          readOnly: true,
        });

        oldMonacoModel = monaco.editor.createModel(data.oldFileContent, "text");
        newMonacoModel = monaco.editor.createModel(data.newFileContent, "text");

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
      })
      .catch((errorUnknown) => {
        let error = errorUnknown as string;

        previewError = error;
      });
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
    editor?.dispose();
    oldMonacoModel?.dispose();
    newMonacoModel?.dispose();
    globalState.lastEditorContent = "";
  });

  let proofFormatOption: "compressed" | "uncompressed" = $state(settingsData.settings.proofFormat);

  let proofFormatOptions: SelectDropdownOption[] = [
    { label: "Uncompressed", value: "uncompressed" },
    { label: "Compressed", value: "compressed" },
  ];

  let firstTimeEffect = true;

  $effect(() => {
    let overrideProofFormat = proofFormatOption;

    if (firstTimeEffect) {
      firstTimeEffect = false;
      return;
    }

    loading = true;
    const text = globalState.lastEditorContent;
    invoke("add_to_database_preview", { text, overrideProofFormat }).then(async (resUnkown) => {
      const data = resUnkown as AddToDatabasePreviewData | null;

      if (data === null) {
        loading = false;
        return;
      }

      addingInvalidHtml = data.invalidHtml;

      oldMonacoModel?.setValue(data.oldFileContent);
      newMonacoModel?.setValue(data.newFileContent);

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

  let addToDatabase = async () => {
    const text = globalState.lastEditorContent;
    invoke("add_to_database", { text, overrideProofFormat: proofFormatOption })
      .then(async (tupleUnknown) => {
        const tuple = tupleUnknown as [AddToDatabaseResult, boolean] | null;

        if (tuple !== null) {
          let [add_to_database_result, redoGrammarCalculations] = tuple;

          if (add_to_database_result.discriminator === "NewHeader") {
            explorerData.addHeader(add_to_database_result.headerPath, add_to_database_result.headerTitle);
          } else if (add_to_database_result.discriminator === "NewStatement") {
            explorerData.addHeaderContent(add_to_database_result.headerPath, add_to_database_result.headerContentI, add_to_database_result.contentRep);

            if (add_to_database_result.contentRep.contentType == "TheoremStatement") {
              globalState.databaseState!.theoremAmount += 1;
            }
          }

          if (redoGrammarCalculations) {
            globalState.databaseState!.grammarCalculationsProgress = 0;
            invoke("perform_grammar_calculations", { databaseId: globalState.databaseState!.databaseId }).then(() => {
              emit("grammar-calculations-performed");
            });
          }

          await tabManager.reloadAllNonEditorTabs();

          await goto("/main");
        }
      })
      .catch((errorUnknown) => {
        addToDatabaseError = errorUnknown as string;
      });
  };
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
              {#if previewError !== null}
                <div class="w-full h-full flex justify-center items-center">
                  <div class="p-2 border rounded-lg max-w-96 text-center">
                    <div class="text-red-600">ERROR</div>
                    {getAddToDatabaseErrorMessage(previewError)}
                  </div>
                </div>
              {:else}
                {#if loading}
                  <div class="w-full h-full flex justify-center items-center">
                    <div>Loading...</div>
                  </div>
                {/if}
                <div class={"px-2 w-full h-full " + (loading ? " invisible " : "")}>
                  <div id="editor-div" class="w-full h-full"></div>
                </div>
              {/if}
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
                    Proof Format:
                    <SelectDropdown bind:value={proofFormatOption} options={proofFormatOptions} disabled={loading}></SelectDropdown>
                  </div>
                  <div class="pt-6">
                    <RoundButton onclick={() => scrollToChange()} additionalClasses="w-full" disabled={loading}>Scroll to Change</RoundButton>
                  </div>
                  <div class="py-2">
                    <hr />
                  </div>
                  {#if addingInvalidHtml}
                    <div class="pt-2">
                      <div class="border rounded-lg p-2">
                        <div class="text-red-600">WARNING</div>
                        <div>You are adding html to the database that could potentially be dangerous. This is checked using a whitelist of tags and attributes that is not exhaustive. If the tags and attributes you are using are guaranteed to be safe, please create an issue on Github so that they can be added to the whitelist.</div>
                      </div>
                    </div>
                  {/if}
                  <div class="pt-2">
                    <RoundButton onclick={addToDatabase} additionalClasses="w-full" disabled={loading}>Confirm Add to Database</RoundButton>
                  </div>
                  {#if addToDatabaseError !== null}
                    <div class="pt-2">
                      <div class="border rounded-lg p-2">
                        <div class="text-red-600">ERROR</div>
                        <div>Something went wrong while adding to the database. Please make sure that the file was not moved or deleted.</div>
                      </div>
                    </div>
                  {/if}
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
