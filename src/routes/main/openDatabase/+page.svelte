<script lang="ts">
  import { goto } from "$app/navigation";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { setEditorSyntaxHighlighting } from "$lib/monaco/monaco";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import type { ColorInformation, HeaderRepresentation, HtmlRepresentation } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let databaseLoaded = $state(false);
  let confirmClicked = false;
  let cancelClicked = false;

  let last_mm_parser_progress = $state(0);

  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    invoke("open_metamath_database", { mmFilePath: globalState.databaseBeingOpened }).then(async () => {
      invoke("perform_grammar_calculations", { mmFilePath: globalState.databaseBeingOpened }).then(() => {
        emit("grammar-calculations-performed");
      });
      // wait 1 second to avoid bug
      await new Promise((r) => setTimeout(r, 1000));
      databaseLoaded = true;
    });

    unlistenFns.push(
      await listen("mm_parser_progress", (e) => {
        let progress = e.payload as number;
        if (progress > last_mm_parser_progress) {
          last_mm_parser_progress = progress;
          document.getElementById("mm_parser_progress_bar")!.style.width = progress + "%";
        }
      })
    );
  });

  onDestroy(() => {
    for (let unlistenFn of unlistenFns) {
      unlistenFn();
    }
  });

  let onCancelClick = async () => {
    if (!cancelClicked) {
      cancelClicked = true;
      globalState.databaseBeingOpened = "";
      await invoke("cancel_open_metamath_database");
      await goto("/main");
    }
  };

  let onConfirmClick = async () => {
    if (!confirmClicked) {
      confirmClicked = true;
      let [topHeaderRep, htmlReps, colorInformation]: [HeaderRepresentation, HtmlRepresentation[], ColorInformation[]] = await invoke("confirm_open_metamath_database");
      explorerData.resetExplorerWithFirstHeader(topHeaderRep);
      htmlData.loadLocal(htmlReps, colorInformation);
      setEditorSyntaxHighlighting(colorInformation);
      emit("mm-db-opened");
      await tabManager.getOpenTab()?.onTabOpen();
      await goto("/main");
      globalState.databaseBeingOpened = "";
    }
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color flex flex-col items-center text-center">
  <div class="w-full text-left py-2">
    <button class="pl-4" onclick={onCancelClick}>{"< Cancel"}</button>
  </div>
  <h1 class="text-3xl py-2">Opening Database</h1>
  <p class="py-2">
    Database path:<br />
    <!-- <span class="border p-1 rounded-lg custom-bg-input-color text-gray-400">{globalState.databaseBeingOpened}</span> -->
    {globalState.databaseBeingOpened}
  </p>
  <div class="my-4">
    Parsing database:
    <div class="mt-2 w-48 border rounded-lg p-2 relative z-0">
      <div class="z-10">Progress: {last_mm_parser_progress}{"%"}</div>
      <div id="mm_parser_progress_bar" class="bg-green-800 absolute start-0 top-0 rounded-lg h-full -z-10"></div>
    </div>
  </div>
  <RoundButton onclick={onConfirmClick} disabled={!databaseLoaded}>Open database</RoundButton>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
