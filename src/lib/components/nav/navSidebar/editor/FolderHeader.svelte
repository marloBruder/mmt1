<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import FolderHeader from "./FolderHeader.svelte";
  import type { Folder, HeaderPath, NameListHeader } from "$lib/sharedState/model.svelte";
  import ChevronDownIcon from "$lib/icons/arrows/ChevronDownIcon.svelte";
  import ChevronRightIcon from "$lib/icons/arrows/ChevronRightIcon.svelte";
  import PlusIcon from "$lib/icons/PlusIcon.svelte";
  import { page } from "$app/stores";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import FileButton from "./FileButton.svelte";
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";

  let { folder, folderPath }: { folder: Folder; folderPath: string } = $props();

  let toggleOpen = async () => {
    if (folder.content === null) {
      await fileExplorerData.openFolder(folder, folderPath);
    } else {
      fileExplorerData.closeFolder(folder);
    }
  };

  let addingSubfolder = $state(false);

  let newSubfolderTitle = $state("");

  $effect(() => {
    if (addingSubfolder) {
      let input = document.getElementById("subheaderName");
      if (input) {
        input.focus();
      }
    }
  });

  let openAddSubfolderInput = async () => {
    if (folder.content == null) {
      await fileExplorerData.openFolder(folder, folderPath);
    }
    addingSubfolder = true;
    newSubfolderTitle = "";
  };

  let addSubfolder = async () => {
    if (newSubfolderTitle === "") {
      // TODO: check whether name exists already
      throw Error("Invalid Name");
    }
    addingSubfolder = false;
    // await invoke("add_header", { title: newSubheaderTitle, insertPath: calcNewPath(header.subHeaders.length) });
    // header.subHeaders.push({ title: newSubheaderTitle, opened: true, theoremNames: [], subHeaders: [] });
  };

  let abortAddingSubfolder = () => {
    addingSubfolder = false;
    newSubfolderTitle = "";
  };

  let onFocusOutSubheaderTitle = async () => {
    if (addingSubfolder) {
      try {
        await addSubfolder();
      } catch (error) {
        abortAddingSubfolder();
      }
    }
  };

  let onkeyDownSubheaderTitle = (event: KeyboardEvent) => {
    if (event.key == "Enter") {
      try {
        addSubfolder();
      } catch (error) {}
    } else if (event.key == "Escape") {
      abortAddingSubfolder();
    }
  };
</script>

<div class="relative h-6 custom-bg-hover-color">
  <button class="h-full w-full text-left absolute" onclick={toggleOpen}>
    <div class="h-6 w-6 div float-left">
      {#if folder.content != null}
        <ChevronDownIcon></ChevronDownIcon>
      {:else}
        <ChevronRightIcon></ChevronRightIcon>
      {/if}
    </div>
    <div class="ml-6 whitespace-nowrap mr-6 overflow-hidden">
      {folder.name}
    </div>
  </button>
  <button aria-label="Add subheader" onclick={openAddSubfolderInput} class="h-4 w-4 absolute bottom-1 end-1">
    <PlusIcon></PlusIcon>
  </button>
</div>
{#if folder.content != null}
  <div class="pl-3">
    {#each folder.content.subfolders as subfolder}
      <FolderHeader folder={subfolder} folderPath={folderPath + subfolder.name + "\\"}></FolderHeader>
    {/each}
    {#if addingSubfolder}
      <input id="subheaderName" type="text" bind:value={newSubfolderTitle} onfocusout={onFocusOutSubheaderTitle} onkeydown={onkeyDownSubheaderTitle} disabled={!addingSubfolder} autocomplete="off" class="disabled:bg-gray-300" />
    {/if}
    {#each folder.content.fileNames as fileName}
      <FileButton {folderPath} {fileName}></FileButton>
    {/each}
  </div>
{/if}
