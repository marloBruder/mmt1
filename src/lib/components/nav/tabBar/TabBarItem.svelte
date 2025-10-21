<script lang="ts">
  import type { Tab } from "$lib/sharedState/tab.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";

  let { tab, index }: { tab: Tab; index: number } = $props();

  let tabClick = (index: number) => {
    tabManager.openTabWithIndex(index);
  };

  let tabClose = async (index: number) => {
    if (tabManager.tabs[index].showUnsavedChanges()) {
      if (!(await confirm("There are unsaved changes in this tab. Are you sure you want to close it?", { okLabel: "Close Tab", kind: "warning" }))) {
        return;
      }
    }

    await tabManager.closeTabWithIndex(index);
  };

  let tabDblClick = (index: number) => {
    tabManager.makeTempTabWithIndexPermanent(index);
  };

  let tabCloseHover = $state(false);

  let tabCloseMouseEnter = () => {
    tabCloseHover = true;
  };

  let tabCloseMouseLeave = () => {
    tabCloseHover = false;
  };
</script>

<div class={"whitespace-nowrap " + (index == tabManager.openTabIndex ? "border-b-2 border-gray-400 " : "")}>
  <button onclick={() => tabClick(index)} ondblclick={() => tabDblClick(index)} class={"h-full px-2 " + (index == tabManager.tempTabIndex ? "italic " : "")}>{tab.name()}</button>
  <button onclick={() => tabClose(index)} onmouseenter={tabCloseMouseEnter} onmouseleave={tabCloseMouseLeave} class="h-full px-2 border-r border-gray-400 -ml-1">
    {#if !tab.showUnsavedChanges() || tabCloseHover}
      X
    {:else}
      •
    {/if}
  </button>
</div>
