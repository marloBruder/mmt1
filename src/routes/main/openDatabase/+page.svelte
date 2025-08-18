<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
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
  let databaseId: number | null = null;
  let confirmClicked = false;
  let cancelClicked = false;

  let invalidHtml: HtmlRepresentation[] = $state([]);
  let invalidHtmlPage: number = $state(0);

  let lastMmParserProgress = $state(0);
  let lastGrammarCalculationsProgress = $state(0);

  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    invoke("open_metamath_database", { mmFilePath: globalState.databaseBeingOpened }).then(async (payload) => {
      [databaseId, invalidHtml] = payload as [number, HtmlRepresentation[]];
      invoke("perform_grammar_calculations", { databaseId }).then(() => {
        emit("grammar-calculations-performed");
      });
      // wait 1 second to avoid bug
      await new Promise((r) => setTimeout(r, 1000));
      databaseLoaded = true;
    });

    unlistenFns.push(
      await listen("mm-parser-progress", (e) => {
        let progress = e.payload as number;
        if (progress > lastMmParserProgress) {
          lastMmParserProgress = progress;
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
      let [databaseId, topHeaderRep, htmlReps, colorInformation]: [number, HeaderRepresentation, HtmlRepresentation[], ColorInformation[]] = await invoke("confirm_open_metamath_database");
      explorerData.resetExplorerWithFirstHeader(topHeaderRep);
      htmlData.loadLocal(htmlReps, colorInformation);
      setEditorSyntaxHighlighting(colorInformation);
      emit("mm-db-opened");
      await tabManager.getOpenTab()?.onTabOpen();
      await goto("/main");
      globalState.databaseBeingOpened = "";
      globalState.databaseState.grammarCalculationsProgress = lastGrammarCalculationsProgress;
      globalState.databaseState.databaseId = databaseId;
    }
  };

  let invalidHtmlPreviousPage = () => {
    invalidHtmlPage -= 1;
  };

  let invalidHtmlNextPage = () => {
    invalidHtmlPage += 1;
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <ScrollableContainer>
    <div class="flex flex-col items-center text-center">
      <div class="w-full text-left py-2">
        <button class="pl-4 disabled:text-gray-700" disabled={!databaseLoaded} onclick={onCancelClick}>{"< Cancel"}</button>
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
      {#if invalidHtml.length != 0}
        <div class="p-2 mx-12 border rounded-lg">
          <h2 class="text-red-600">WARNING</h2>
          The HTML representation of symbols in this database does not follow all rules for safe HTML checked by mmt1. The following
          <span class="text-red-600">{invalidHtml.length}</span>
          rules may be dangerous. This does not mean that they must be dangerous, but that they could be. Please manually check that
          <span class="text-red-600">EVERY SINGLE</span>
          one of them is safe:
          <div class="mt-4">
            <table class=" mx-auto">
              <thead>
                <tr>
                  <th></th>
                  <th class="border">Symbol</th>
                  <th class="border">HTML representation</th>
                </tr>
              </thead>
              <tbody>
                {#each invalidHtml as invalidHtmlRep, i}
                  {#if invalidHtmlPage * 10 <= i && i < (invalidHtmlPage + 1) * 10}
                    <tr>
                      <td class="border">{i + 1}</td>
                      <td class="border">{invalidHtmlRep.symbol}</td>
                      <td class="border">{invalidHtmlRep.html}</td>
                    </tr>
                  {/if}
                {/each}
              </tbody>
            </table>
            <div class="flex flex-row justify-center mt-2">
              <div class="px-2">
                <RoundButton onclick={invalidHtmlPreviousPage} disabled={invalidHtmlPage === 0}>Previous Page</RoundButton>
              </div>
              <div class="px-2">
                <RoundButton onclick={invalidHtmlNextPage} disabled={invalidHtmlPage >= Math.floor((invalidHtml.length - 1) / 10)}>Next Page</RoundButton>
              </div>
            </div>
          </div>
          If you have a tag or attribute that is guaranteed to be safe, but that is not on the whitelist, please create an issue on Github, so it can be added to the rules of what makes an html representation safe.
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
