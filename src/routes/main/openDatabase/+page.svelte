<script lang="ts">
  import { goto } from "$app/navigation";
  import InvalidHtmlPopup from "$lib/components/util/InvalidHtmlPopup.svelte";
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import { setEditorSyntaxHighlighting } from "$lib/monaco/monaco";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { DatabaseState, globalState } from "$lib/sharedState/globalState.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import type { ColorInformation, HeaderRepresentation, HtmlRepresentation } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let databaseLoaded = $state(false);
  let cancelable = $state(false);
  let databaseId: number | null = null;
  let confirmClicked = false;
  let cancelClicked = false;

  let invalidHtml: HtmlRepresentation[] = $state([]);
  let invalidDescriptionHtml: [string, string][] = $state([]);

  let lastMmParserProgress = $state(0);
  let lastCalcOptimizedTheoremDataProgress = $state(0);
  let lastGrammarCalculationsProgress = $state(0);

  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    unlistenFns.push(
      await listen("mm-parser-progress", (e) => {
        let progress = e.payload as number;
        if (progress > lastMmParserProgress) {
          lastMmParserProgress = progress;
        }
      })
    );
    unlistenFns.push(
      await listen("calc-optimized-theorem-data-progress", (e) => {
        let progress = e.payload as number;
        if (progress > lastCalcOptimizedTheoremDataProgress) {
          lastCalcOptimizedTheoremDataProgress = progress;
        }
      })
    );
    unlistenFns.push(
      await listen("grammar-calculations-progress", (e) => {
        let [progress, id] = e.payload as [number, number];
        if (databaseId === id && progress > lastGrammarCalculationsProgress) {
          lastGrammarCalculationsProgress = progress;
        }
      })
    );

    invoke("open_metamath_database", { mmFilePath: globalState.databaseBeingOpened }).then(async (payload) => {
      [databaseId, invalidHtml, invalidDescriptionHtml] = payload as [number, HtmlRepresentation[], [string, string][]];
      invoke("perform_grammar_calculations", { databaseId }).then(() => {
        emit("grammar-calculations-performed");
      });
      // wait 1 second to avoid bug
      await new Promise((r) => setTimeout(r, 1000));
      databaseLoaded = true;
    });

    cancelable = true;
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
      let [databaseId, theoremAmount, topHeaderRep, htmlReps, colorInformation]: [number, number, HeaderRepresentation, HtmlRepresentation[], ColorInformation[]] = await invoke("confirm_open_metamath_database");
      explorerData.resetExplorerWithFirstHeader(topHeaderRep);
      htmlData.loadLocal(htmlReps, colorInformation);
      setEditorSyntaxHighlighting(colorInformation);
      emit("mm-db-opened");
      await tabManager.getOpenTab()?.onTabOpen();
      await goto("/main");
      globalState.databaseState = new DatabaseState(databaseId, globalState.databaseBeingOpened, theoremAmount);
      globalState.databaseState.grammarCalculationsProgress = lastGrammarCalculationsProgress;
      globalState.databaseBeingOpened = "";
    }
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <ScrollableContainer>
    <div class="flex flex-col items-center text-center">
      <div class="w-full text-left py-2">
        <button class="pl-4 disabled:text-gray-700" disabled={!cancelable} onclick={onCancelClick}>{"< Cancel"}</button>
      </div>
      <h1 class="text-3xl py-2">Opening Database</h1>
      <p class="py-2">
        Database path:<br />
        <!-- <span class="border p-1 rounded-lg custom-bg-input-color text-gray-400">{globalState.databaseBeingOpened}</span> -->
        {globalState.databaseBeingOpened}
      </p>
      <div class="my-4">
        Parsing database:
        <ProgressBar progress={lastMmParserProgress}></ProgressBar>
      </div>
      <InvalidHtmlPopup invalidHtml={invalidHtml.map((htmlRep) => [htmlRep.symbol, htmlRep.html])}></InvalidHtmlPopup>
      <div class="my-4">
        Calculating relevant theorem data:
        <div class="flex flex-col items-center">
          <div>
            <ProgressBar progress={lastCalcOptimizedTheoremDataProgress}></ProgressBar>
          </div>
        </div>
      </div>
      <InvalidHtmlPopup invalidHtml={invalidDescriptionHtml} descriptionHtml></InvalidHtmlPopup>
      {#if invalidHtml.length !== 0 || invalidDescriptionHtml.length !== 0}
        <div class="border rounded-lg p-2 mx-12 my-4">
          <h2 class="text-blue-400">INFO</h2>
          The whitelist for what is considered safe HTML was created based on what is in set.mm and is by no means exhaustive. If you have a tag or attribute that is guaranteed to be safe, but that is not on the whitelist, please create an issue on Github, so it can be added to the rules of what makes HTML safe.
        </div>
      {/if}
      <div class="mt-4">
        <RoundButton onclick={onConfirmClick} disabled={!databaseLoaded}>Open database</RoundButton>
      </div>
      <div class="my-4">
        Calculating parse trees:
        <ProgressBar progress={lastGrammarCalculationsProgress}></ProgressBar>
      </div>
    </div>
  </ScrollableContainer>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
