<script lang="ts">
  import EmptyTabComponent from "$lib/components/tabs/EmptyTabComponent.svelte";
  import { emit, listen } from "@tauri-apps/api/event";
  import type { DatabaseElementPageData, HtmlRepresentation } from "$lib/sharedState/model.svelte";
  import type { Tab } from "$lib/sharedState/tabManager.svelte";
  import { onMount } from "svelte";
  import EditorTabSplitViewComponent from "$lib/components/tabs/EditorTabSplitViewComponent.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import TitleBar from "$lib/components/titleBar/TitleBar.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let pageDataLoaded: boolean = $state(false);
  let pageData: DatabaseElementPageData | null = $state(null);

  onMount(() => {
    listen("mm-db-opened", () => {
      htmlData.load();
    });

    htmlData.load();

    // If event "external-window-close" has been triggered and the window is still up, then close the external window
    listen("external-window-close", () => {
      getCurrentWindow().close();
    });

    invoke("set_up_external_window_close_listener");

    listen("split-view-page-data-transfer", (event) => {
      pageDataLoaded = true;
      pageData = event.payload as DatabaseElementPageData | null;
    });

    emit("request-first-split-view-transfer");
  });
</script>

<div class="h-screen w-screen fixed custom-bg-bg-color text-gray-300">
  <HorizontalSplit>
    {#snippet first()}
      <div class="h-8 w-full overflow-hidden">
        <TitleBar externalWindow></TitleBar>
      </div>
    {/snippet}
    {#snippet second()}
      <div class="custom-height-minus-margin custom-width-minus-margin custom-bg-color m-2 rounded-lg">
        {#if pageDataLoaded}
          <EditorTabSplitViewComponent {pageData}></EditorTabSplitViewComponent>
        {/if}
      </div>
    {/snippet}
  </HorizontalSplit>
</div>

<style>
  .custom-height-minus-margin {
    height: calc(100% - 1rem);
  }

  .custom-width-minus-margin {
    width: calc(100% - 1rem);
  }
</style>
