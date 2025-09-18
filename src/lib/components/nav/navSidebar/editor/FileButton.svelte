<script lang="ts">
  import { EditorTab } from "$lib/components/tabs/EditorTabComponent.svelte";
  import ContextMenuElement from "$lib/components/util/ContextMenuElement.svelte";
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { confirm } from "@tauri-apps/plugin-dialog";

  let { folderPath, fileName, reloadFolder, nameExists }: { folderPath: string; fileName: string; reloadFolder: () => void; nameExists: (name: string) => boolean } = $props();

  let explorerClick = () => {
    tabManager.openTab(new EditorTab(folderPath + fileName));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new EditorTab(folderPath + fileName));
  };

  let openFile = () => {
    tabManager.openTab(new EditorTab(folderPath + fileName), true);
  };

  let copyPath = async () => {
    let openedFolderPath = (await invoke("get_opened_folder_path")) as string;
    navigator.clipboard.writeText(openedFolderPath + "\\" + folderPath + fileName);
  };

  let copyRelativePath = () => {
    navigator.clipboard.writeText(folderPath + fileName);
  };

  let renaming = $state(false);
  let renameInputValue = $state(fileName);
  let renameInputId = "file-button-rename-input-" + createInstanceId();
  let renameErrorWarning = $state(false);

  let rename = () => {
    renaming = true;
  };

  $effect(() => {
    if (renaming) {
      let inputElement = document.getElementById(renameInputId);
      if (inputElement !== null) {
        inputElement.focus();
      }
    }
  });

  let renameInputOnFocusOut = () => {
    if (!nameExists(renameInputValue) || renameInputValue === fileName) {
      confirmRename();
    } else {
      abortRename();
    }
  };

  let renameInputOnkeydown = (e: KeyboardEvent) => {
    renameErrorWarning = false;
    if (e.key === "Escape") {
      abortRename();
    } else if (e.key === "Enter") {
      confirmRename();
    }
  };

  let confirmRename = async () => {
    if (renaming) {
      if (!nameExists(renameInputValue) || renameInputValue === fileName) {
        renaming = false;
        let newFileName = renameInputValue;
        renameInputValue = fileName;
        renameErrorWarning = false;
        if (fileName !== newFileName) {
          let successful = (await invoke("rename_file", { folderPath, fileName, newFileName })) as boolean;
          if (successful) {
            reloadFolder();
            tabManager.tabs.forEach((tab) => {
              if (tab instanceof EditorTab && tab.filePath === folderPath + fileName) {
                tab.filePath = newFileName;
              }
            });
          }
        }
      } else {
        renameErrorWarning = true;
      }
    }
  };

  let abortRename = () => {
    renaming = false;
    renameInputValue = fileName;
    renameErrorWarning = false;
  };

  let deleteFile = async () => {
    if (await confirm("Are you sure you want to delete '" + fileName + "'?", { okLabel: "Delete" })) {
      let successful = (await invoke("delete_file", { relativePath: folderPath + fileName })) as boolean;
      if (successful) {
        reloadFolder();
        tabManager.closeTabsWithCondition((tab) => {
          return tab instanceof EditorTab && tab.filePath === folderPath + fileName;
        });
      }
    }
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div>
      {#if !renaming}
        <button class="w-full text-left pl-2 custom-bg-hover-color text-nowrap" onclick={explorerClick} ondblclick={explorerDblClick}>{fileName}</button>
      {:else}
        <input id={renameInputId} bind:value={renameInputValue} class={"w-full custom-bg-input-color " + (renameErrorWarning ? " border border-red-500 " : "")} onfocusout={renameInputOnFocusOut} onkeydown={renameInputOnkeydown} autocomplete="off" spellcheck="false" />
        <div class={"fixed max-w-44 text-xs bg-red-500 " + (renameErrorWarning ? "" : " invisible ")}>A file or folder with this name already exists at this location. Please choose a different name.</div>
      {/if}
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={openFile}>Open</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={copyPath}>Copy Path</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={copyRelativePath}>Copy Relative Path</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={rename}>Rename</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500" onclick={deleteFile}>Delete</button></div>
  {/snippet}
</ContextMenuElement>
