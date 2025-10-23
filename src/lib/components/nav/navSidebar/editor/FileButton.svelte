<script lang="ts">
  import { EditorTab } from "$lib/components/tabs/EditorTabComponent.svelte";
  import ContextMenuButton from "$lib/components/util/contextMenu/ContextMenuButton.svelte";
  import ContextMenuDivider from "$lib/components/util/contextMenu/ContextMenuDivider.svelte";
  import ContextMenuElement from "$lib/components/util/contextMenu/ContextMenuElement.svelte";
  import HiddenInput from "$lib/components/util/HiddenInput.svelte";
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

  let rename = () => {
    renaming = true;
  };

  let validNewName = (name: string) => !nameExists(name) || name === fileName;

  let onRenameConfirm = async (newValue: string) => {
    let successful = (await invoke("rename_file", { folderPath, fileName, newFileName: newValue })) as boolean;
    if (successful) {
      reloadFolder();
      tabManager.tabs.forEach((tab) => {
        if (tab instanceof EditorTab && tab.filePath === folderPath + fileName) {
          tab.filePath = folderPath + newValue;
        }
      });
    }
  };

  let deleteFile = async () => {
    if (await confirm("Are you sure you want to delete '" + fileName + "'? This is irreversible!", { okLabel: "Delete" })) {
      let successful = (await invoke("delete_file", { relativePath: folderPath + fileName })) as boolean;
      if (successful) {
        reloadFolder();
        await tabManager.closeTabsWithCondition((tab) => {
          return tab instanceof EditorTab && tab.filePath === folderPath + fileName;
        });
      }
    }
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div class={"pl-2 " + (renaming ? "" : " custom-bg-hover-color ")}>
      <HiddenInput bind:visible={renaming} validInput={validNewName} previousValue={fileName} onconfirm={onRenameConfirm}>
        <button class="w-full text-left text-nowrap overflow-hidden" onclick={explorerClick} ondblclick={explorerDblClick}>{fileName}</button>
      </HiddenInput>
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <ContextMenuButton onclick={openFile}>Open</ContextMenuButton>
    <ContextMenuDivider></ContextMenuDivider>
    <ContextMenuButton onclick={copyPath}>Copy Path</ContextMenuButton>
    <ContextMenuButton onclick={copyRelativePath}>Copy Relative Path</ContextMenuButton>
    <ContextMenuDivider></ContextMenuDivider>
    <ContextMenuButton onclick={rename}>Rename</ContextMenuButton>
    <ContextMenuButton onclick={deleteFile}>Delete</ContextMenuButton>
  {/snippet}
</ContextMenuElement>
