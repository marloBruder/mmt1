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
  import { emit } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let databaseLoaded = $state(false);
  let confirmClicked = false;
  let cancelClicked = false;

  onMount(() => {
    invoke("open_metamath_database", { mmFilePath: globalState.databaseBeingOpened }).then(async () => {
      invoke("perform_grammar_calculations", { mmFilePath: globalState.databaseBeingOpened }).then(() => {
        emit("grammar-calculations-performed");
      });
      // wait 1 second to avoid bug
      await new Promise((r) => setTimeout(r, 1000));
      databaseLoaded = true;
    });
  });

  let onCancelClick = async () => {
    if (!cancelClicked) {
      cancelClicked = true;
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

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color text-center">
  <div class="text-left py-2">
    <button class="pl-4" onclick={onCancelClick}>{"< Cancel"}</button>
  </div>
  <h1 class="text-3xl py-2">Opening Database</h1>
  <p class="py-2">
    Database path:
    <!-- <span class="border p-1 rounded-lg custom-bg-input-color text-gray-400">{globalState.databaseBeingOpened}</span> -->
    {globalState.databaseBeingOpened}
  </p>
  <RoundButton onclick={onConfirmClick} disabled={!databaseLoaded}>Open database</RoundButton>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
