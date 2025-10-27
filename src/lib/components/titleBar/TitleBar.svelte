<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { invoke } from "@tauri-apps/api/core";
  import { confirm, open, save } from "@tauri-apps/plugin-dialog";
  import type { FolderRepresentation, HeaderRepresentation } from "$lib/sharedState/model.svelte";
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";
  import CloseIcon from "$lib/icons/titleBar/CloseIcon.svelte";
  import MaximizeIcon from "$lib/icons/titleBar/MaximizeIcon.svelte";
  import MinimizeIcon from "$lib/icons/titleBar/MinimizeIcon.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import UnMaximizeIcon from "$lib/icons/titleBar/UnMaximizeIcon.svelte";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { DatabaseState, globalState } from "$lib/sharedState/globalState.svelte";
  import Dropdown from "../util/Dropdown.svelte";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import { searchData } from "$lib/sharedState/searchData.svelte";
  import ContextMenuButton from "../util/contextMenu/ContextMenuButton.svelte";
  import ContextMenuDivider from "../util/contextMenu/ContextMenuDivider.svelte";

  let { externalWindow = false }: { externalWindow?: boolean } = $props();

  const appWindow = getCurrentWindow();

  let disableTitleBar = $derived(globalState.databaseBeingOpened !== "");

  let dropdown1Open = $state(false);
  let dropdown2Open = $state(false);
  let dropdown3Open = $state(false);
  let dropdown4Open = $state(false);

  let minimizeClick = () => {
    appWindow.minimize();
  };

  let maximizeClick = () => {
    appWindow.toggleMaximize();
  };

  let closeClick = async () => {
    if (tabManager.tabs.some((tab) => tab.showUnsavedChanges())) {
      if (!(await confirm("You have unsaved changes. Are you sure you want to close mmt1?", { okLabel: "Close mmt1", kind: "warning" }))) {
        return;
      }
    }

    appWindow.close();
  };

  let onOpenFolderClick = async () => {
    const folderPath = await open({ multiple: false, directory: true });

    if (folderPath) {
      let folderRep = (await invoke("open_folder", { folderPath })) as FolderRepresentation;
      let folderPathSplit = folderPath.split(/[/\\]/);
      fileExplorerData.resetDataWithFirstFolder(folderPathSplit[folderPathSplit.length - 1], folderRep);
    }
  };

  let onCloseFolderClick = async () => {
    await invoke("close_folder");
    fileExplorerData.resetData();
  };

  let onSaveFileClick = async () => {
    tabManager.getOpenTab()!.saveFile();
  };

  let onUnifyClick = async () => {
    tabManager.getOpenTab()!.unify();
  };

  let onFormatClick = async () => {
    tabManager.getOpenTab()!.format();
  };

  let onRenumberClick = async () => {
    tabManager.getOpenTab()!.renumber();
  };

  let onAddToDatabaseClick = () => {
    tabManager.getOpenTab()!.addToDatabase();
  };

  let onNewMetamathDatabaseClick = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

    if (filePath) {
      let [headerRep, databaseId] = (await invoke("new_database", { filePath })) as [HeaderRepresentation, number];
      await tabManager.closeAllNonEditorOrSettingsTabs();
      explorerData.resetExplorerWithFirstHeader(headerRep);
      globalState.databaseState = new DatabaseState(databaseId, filePath, 0);
      htmlData.loadLocal([], []);
      searchData.resetSearchParameters();
    }
  };

  let onOpenMetamathDatabaseClick = async () => {
    const filePath = await open({ multiple: false, directory: false, filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

    if (filePath) {
      globalState.databaseBeingOpened = filePath;
      await goto("/main/openDatabase");
      // let [topHeaderRep, htmlReps, colorInformation]: [HeaderRepresentation, HtmlRepresentation[], ColorInformation[]] = await invoke("open_metamath_database", { mmFilePath: filePath });
      // explorerData.resetExplorerWithFirstHeader(topHeaderRep);
      // htmlData.loadLocal(htmlReps, colorInformation);
      // setEditorSyntaxHighlighting(colorInformation);
      // emit("mm-db-opened");
    }
  };

  // let onExportMetamathDatabaseClick = async () => {
  //   const filePath = await save({ filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

  //   if (filePath) {
  //     await invoke("export_database", { filePath });
  //   }
  // };

  let onCloseMetamathDatabaseClick = async () => {
    await invoke("close_metamath_database");
    await tabManager.closeAllNonEditorOrSettingsTabs();
    explorerData.resetExplorer();
    globalState.databaseState = null;
    htmlData.loadLocal([], []);
    searchData.resetSearchParameters();
  };

  let onAboutClick = () => {
    goto("/main/about");
  };

  let onGuidesClick = () => {
    goto("/main/guides");
  };

  let onCheckForUpdatesClick = () => {
    goto("/main/updates");
  };

  let isMaximized = $state(true);

  appWindow.onResized(async () => {
    isMaximized = await appWindow.isMaximized();
  });

  onMount(async () => {
    isMaximized = await appWindow.isMaximized();
  });

  let onmouseenterDropdownButton = (num: number) => {
    if ([dropdown1Open, dropdown2Open, dropdown3Open, dropdown4Open].some((bool) => bool)) {
      // reuse function to set everything to false;
      customDropdownOnclose();
      switch (num) {
        case 0:
          dropdown1Open = true;
          break;
        case 1:
          dropdown2Open = true;
          break;
        case 2:
          dropdown3Open = true;
          break;
        case 3:
          dropdown4Open = true;
          break;
      }
    }
  };

  let customDropdownOnclose = () => {
    dropdown1Open = false;
    dropdown2Open = false;
    dropdown3Open = false;
    dropdown4Open = false;
  };
</script>

<div class="h-8 w-screen flex justify-between" data-tauri-drag-region>
  <div class="pl-4 h-full flex items-center">
    <span class="text-xl pr-2">mmt1</span>
    {#if !externalWindow}
      <Dropdown title="File" disabled={disableTitleBar} bind:open={dropdown1Open} onmouseenter={() => onmouseenterDropdownButton(0)} customOnclose={customDropdownOnclose}>
        {#snippet dropdownContent()}
          <ContextMenuButton onclick={onOpenFolderClick}>Open Folder</ContextMenuButton>
          <ContextMenuButton onclick={onCloseFolderClick}>Close Folder</ContextMenuButton>
          <ContextMenuDivider></ContextMenuDivider>
          <ContextMenuButton onclick={onSaveFileClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.saveFileDisabled() : true}>Save File</ContextMenuButton>
          <ContextMenuDivider></ContextMenuDivider>
          <ContextMenuButton onclick={closeClick}>Exit</ContextMenuButton>
        {/snippet}
      </Dropdown>
      <Dropdown title="Editor" disabled={disableTitleBar} bind:open={dropdown2Open} onmouseenter={() => onmouseenterDropdownButton(1)} customOnclose={customDropdownOnclose}>
        {#snippet dropdownContent()}
          <ContextMenuButton onclick={onUnifyClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.unifyDisabled() : true}>Unify</ContextMenuButton>
          <ContextMenuButton onclick={onFormatClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.formatDisabled() : true}>Format</ContextMenuButton>
          <ContextMenuButton onclick={onRenumberClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.renumberDisabled() : true}>Renumber</ContextMenuButton>
          <ContextMenuDivider></ContextMenuDivider>
          <ContextMenuButton onclick={onAddToDatabaseClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.addToDatabaseDisabled() : true}>Add to database</ContextMenuButton>
        {/snippet}
      </Dropdown>
      <Dropdown title="Metamath" disabled={disableTitleBar} bind:open={dropdown3Open} onmouseenter={() => onmouseenterDropdownButton(2)} customOnclose={customDropdownOnclose}>
        {#snippet dropdownContent()}
          <ContextMenuButton onclick={onNewMetamathDatabaseClick}>New Metamath Database</ContextMenuButton>
          <ContextMenuButton onclick={onOpenMetamathDatabaseClick}>Open Metamath Database</ContextMenuButton>
          <!-- <div><button class="hover:bg-purple-500 px-2 w-full text-left" onclick={onExportMetamathDatabaseClick}>Export Metamath Database</button></div> -->
          <ContextMenuButton onclick={onCloseMetamathDatabaseClick} disabled={globalState.databaseState === null}>Close Metamath Database</ContextMenuButton>
        {/snippet}
      </Dropdown>
      <Dropdown title="About" disabled={disableTitleBar} bind:open={dropdown4Open} onmouseenter={() => onmouseenterDropdownButton(3)} customOnclose={customDropdownOnclose}>
        {#snippet dropdownContent()}
          <ContextMenuButton onclick={onAboutClick}>About mmt1</ContextMenuButton>
          <ContextMenuButton onclick={onGuidesClick}>Guides</ContextMenuButton>
          <ContextMenuButton onclick={onCheckForUpdatesClick}>Check For Updates</ContextMenuButton>
        {/snippet}
      </Dropdown>
    {:else}
      <div class="pl-2">Editor HTML Preview</div>
    {/if}
  </div>
  <div class="flex">
    <button class="px-3 hover:bg-gray-700" onclick={minimizeClick}><MinimizeIcon /></button>
    <button class="px-3 hover:bg-gray-700" onclick={maximizeClick}>
      {#if isMaximized}
        <UnMaximizeIcon />
      {:else}
        <MaximizeIcon />
      {/if}
    </button>
    <button class="px-3 hover:bg-red-700" onclick={closeClick}><CloseIcon /></button>
  </div>
</div>
