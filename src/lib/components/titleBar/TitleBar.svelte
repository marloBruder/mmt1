<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TitleBarDropdown from "./TitleBarDropdown.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import type { ColorInformation, FolderRepresentation, HeaderRepresentation, HtmlRepresentation } from "$lib/sharedState/model.svelte";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";
  import CloseIcon from "$lib/icons/titleBar/CloseIcon.svelte";
  import MaximizeIcon from "$lib/icons/titleBar/MaximizeIcon.svelte";
  import MinimizeIcon from "$lib/icons/titleBar/MinimizeIcon.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { setSyntaxHighlighting } from "$lib/monaco/monaco";
  import UnMaximizeIcon from "$lib/icons/titleBar/UnMaximizeIcon.svelte";

  const appWindow = getCurrentWindow();

  let minimizeClick = () => {
    appWindow.minimize();
  };

  let maximizeClick = () => {
    appWindow.toggleMaximize();
  };

  let closeClick = () => {
    appWindow.close();
  };

  let onOpenFolderClick = async () => {
    const folderPath = await open({ multiple: false, directory: true });

    if (folderPath) {
      let folderRep = (await invoke("open_folder", { folderPath })) as FolderRepresentation;
      let folderPathSplit = folderPath.split("\\");
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

  let onNewMetamathDatabaseClick = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

    if (filePath) {
      await invoke("new_database", { filePath });
    }
  };

  let onOpenMetamathDatabaseClick = async () => {
    const filePath = await open({ multiple: false, directory: false, filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

    if (filePath) {
      let [topHeaderRep, htmlReps, colorInformation]: [HeaderRepresentation, HtmlRepresentation[], ColorInformation[]] = await invoke("open_metamath_database", { mmFilePath: filePath });
      explorerData.resetExplorerWithFirstHeader(topHeaderRep);
      htmlData.loadLocal(htmlReps);
      setSyntaxHighlighting(colorInformation);
    }
  };

  let onExportMetamathDatabaseClick = async () => {
    const filePath = await save({ filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

    if (filePath) {
      await invoke("export_database", { filePath });
    }
  };

  let onAddToDatabaseClick = () => {
    tabManager.getOpenTab()!.addToDatabase();
  };

  let isMaximized = $state(false);

  appWindow.onResized(async () => {
    isMaximized = await appWindow.isMaximized();
  });
</script>

<div class="h-8 w-screen flex justify-between" data-tauri-drag-region>
  <div class="pl-4 h-full flex items-center">
    <span class="text-xl pr-2">mmt1</span>
    <TitleBarDropdown title="File">
      <div><button onclick={onOpenFolderClick}>Open Folder</button></div>
      <div><button onclick={onCloseFolderClick}>Close Folder</button></div>
      <hr class="border-gray-300" />
      <div><button onclick={onSaveFileClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.saveFileDisabled() : true} class="disabled:text-gray-500">Save File</button></div>
      <hr class="border-gray-300" />
      <div><button onclick={closeClick}>Exit</button></div>
    </TitleBarDropdown>
    <TitleBarDropdown title="Unify">
      <div><button onclick={onUnifyClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.unifyDisabled() : true} class="disabled:text-gray-500">Unify</button></div>
    </TitleBarDropdown>
    <TitleBarDropdown title="Metamath">
      <div><button onclick={onNewMetamathDatabaseClick}>New Metamath Database</button></div>
      <div><button onclick={onOpenMetamathDatabaseClick}>Open Metamath Database</button></div>
      <div><button onclick={onExportMetamathDatabaseClick}>Export Metamath Database</button></div>
      <hr class="border-gray-300" />
      <div><button onclick={onAddToDatabaseClick} disabled={tabManager.getOpenTab() ? tabManager.getOpenTab()!.addToDatabaseDisabled() : true} class="disabled:text-gray-500">Add to database</button></div>
    </TitleBarDropdown>
  </div>
  <div class="flex">
    <button class="mx-3" onclick={minimizeClick}><MinimizeIcon /></button>
    <button class="mx-3" onclick={maximizeClick}>
      {#if isMaximized}
        <UnMaximizeIcon />
      {:else}
        <MaximizeIcon />
      {/if}
    </button>
    <button class="mx-3" onclick={closeClick}><CloseIcon /></button>
  </div>
</div>
