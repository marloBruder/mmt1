<script lang="ts">
  import FolderHeader from "./FolderHeader.svelte";
  import type { Folder } from "$lib/sharedState/model.svelte";
  import ChevronDownIcon from "$lib/icons/arrows/ChevronDownIcon.svelte";
  import ChevronRightIcon from "$lib/icons/arrows/ChevronRightIcon.svelte";
  import FileButton from "./FileButton.svelte";
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";
  import ContextMenuElement from "$lib/components/util/ContextMenuElement.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import HiddenInput from "$lib/components/util/HiddenInput.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { EditorTab } from "$lib/components/tabs/EditorTabComponent.svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";

  let {
    folder,
    folderPath,
    folderName,
    nameExistsInParent,
    reloadParentFolder,
    noRenameOrDelete = false,
  }: {
    folder: Folder;
    folderPath: string;
    folderName: string;
    nameExistsInParent: (name: string) => boolean;
    reloadParentFolder: (rename?: [string, string]) => void;
    noRenameOrDelete?: boolean;
  } = $props();

  let toggleOpen = async () => {
    if (folder.content === null) {
      await fileExplorerData.openFolder(folder, folderPath);
    } else {
      fileExplorerData.closeFolder(folder);
    }
  };

  let addingSubfolder = $state(false);

  let openAddSubfolderInput = async () => {
    if (folder.content == null) {
      await fileExplorerData.openFolder(folder, folderPath);
    }
    addingSubfolder = true;
  };

  let validNewName = (name: string) => !nameExists(name);

  let addSubfolder = async (newSubfolderName: string) => {
    let successful = (await invoke("create_folder", { relativePath: folderPath + newSubfolderName })) as boolean;
    if (successful) {
      await fileExplorerData.reloadFolder(folder, folderPath);
    }
  };

  let addingFile = $state(false);

  let openAddFileInput = async () => {
    if (folder.content == null) {
      await fileExplorerData.openFolder(folder, folderPath);
    }
    addingFile = true;
  };

  let addFile = async (newFileName: string) => {
    let successful = (await invoke("create_file", { relativePath: folderPath + newFileName })) as boolean;
    if (successful) {
      await fileExplorerData.reloadFolder(folder, folderPath);
      await tabManager.openTab(new EditorTab(folderPath + newFileName), true);
    }
  };

  let copyPath = async () => {
    let openedFolderPath = (await invoke("get_opened_folder_path")) as string;
    navigator.clipboard.writeText(openedFolderPath + "\\" + folderPath);
  };

  let copyRelativePath = () => {
    navigator.clipboard.writeText(folderPath);
  };

  let renaming = $state(false);

  let openRenameInput = async () => {
    renaming = true;
  };

  let validNewNameInParent = (name: string) => !nameExistsInParent(name);

  let renameFolder = async (newFolderName: string) => {
    let [successful, filePathRenames] = (await invoke("rename_folder", { folderPath, newFolderName })) as [boolean, [string, string][]];
    if (successful) {
      reloadParentFolder([folderName, newFolderName]);
      tabManager.tabs.forEach((tab) => {
        if (tab instanceof EditorTab) {
          let rename = filePathRenames.find(([oldPath, _]) => oldPath === tab.filePath);
          if (rename !== undefined) {
            let [_, newPath] = rename;
            tab.filePath = newPath;
          }
        }
      });
    }
  };

  let deleteFolder = async () => {
    let openedFolderPath = (await invoke("get_opened_folder_path")) as string;
    if (await confirm("Are you sure you want to delete '" + openedFolderPath + "\\" + folderPath + "'? This is irreversible!", { okLabel: "Delete" })) {
      let successful = (await invoke("delete_folder", { relativePath: folderPath })) as boolean;
      if (successful) {
        reloadParentFolder();
        await tabManager.closeTabsWithCondition((tab) => {
          return tab instanceof EditorTab && tab.filePath.startsWith(folderPath);
        });
      }
    }
  };

  let reloadFolder = (rename?: [string, string]) => {
    if (rename === undefined) {
      fileExplorerData.reloadFolder(folder, folderPath);
    } else {
      let [oldFolderName, newFolderName] = rename;
      fileExplorerData.reloadFolderWithRename(folder, folderPath, oldFolderName, newFolderName);
    }
  };

  let nameExists = (name: string): boolean => {
    if (folder.content !== null) {
      return folder.content.fileNames.some((fileName) => fileName === name) || folder.content.subfolders.some((subfolder) => subfolder.name === name);
    } else {
      return false;
    }
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div class="h-6 custom-bg-hover-color">
      <button class="h-full w-full text-left flex flex-row" onclick={toggleOpen}>
        <div class="h-6 w-6">
          {#if folder.content != null}
            <ChevronDownIcon></ChevronDownIcon>
          {:else}
            <ChevronRightIcon></ChevronRightIcon>
          {/if}
        </div>
        <div class="text-nowrap overflow-hidden">
          <HiddenInput bind:visible={renaming} previousValue={folderName} validInput={validNewNameInParent} onconfirm={renameFolder}>
            {folder.name}
          </HiddenInput>
        </div>
      </button>
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={openAddFileInput}>New File</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={openAddSubfolderInput}>New Folder</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={copyPath}>Copy Path</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={copyRelativePath}>Copy Relative Path</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:enabled:bg-purple-500 disabled:text-gray-400" onclick={openRenameInput} disabled={noRenameOrDelete}>Rename</button></div>
    <div><button class="w-full px-2 text-left hover:enabled:bg-purple-500 disabled:text-gray-400" onclick={deleteFolder} disabled={noRenameOrDelete}>Delete</button></div>
  {/snippet}
</ContextMenuElement>
{#if folder.content != null}
  <div class="pl-3">
    <HiddenInput bind:visible={addingSubfolder} previousValue="" validInput={validNewName} onconfirm={addSubfolder}></HiddenInput>
    {#each folder.content.subfolders as subfolder (subfolder.name)}
      <FolderHeader folder={subfolder} folderPath={folderPath + subfolder.name + "\\"} folderName={subfolder.name} reloadParentFolder={reloadFolder} nameExistsInParent={nameExists}></FolderHeader>
    {/each}
    <HiddenInput bind:visible={addingFile} previousValue="" validInput={validNewName} onconfirm={addFile}></HiddenInput>
    {#each folder.content.fileNames as fileName (fileName)}
      <FileButton {folderPath} {fileName} {reloadFolder} {nameExists}></FileButton>
    {/each}
  </div>
{/if}
