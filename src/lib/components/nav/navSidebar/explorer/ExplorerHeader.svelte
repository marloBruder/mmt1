<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerHeader from "./ExplorerHeader.svelte";
  import { goto } from "$app/navigation";
  import type { HeaderPath, HeaderRepresentation, NameListHeader } from "$lib/sharedState/model.svelte";
  import ChevronDownIcon from "$lib/icons/ChevronDownIcon.svelte";
  import ChevronRightIcon from "$lib/icons/ChevronRightIcon.svelte";
  import PlusIcon from "$lib/icons/PlusIcon.svelte";
  import { page } from "$app/stores";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";

  let { header, headerPath }: { header: NameListHeader; headerPath: HeaderPath } = $props();

  let pathString = $derived.by(() => {
    let stringRep = "";
    for (let pos of headerPath.path) {
      stringRep = stringRep + (pos + 1) + ".";
    }
    stringRep = stringRep.slice(0, stringRep.length - 1);
    return stringRep;
  });

  let calcNewPath = (index: number): HeaderPath => {
    let newPath = { path: headerPath.path.slice() };
    newPath.path.push(index);
    return newPath;
  };

  let toggleOpen = async () => {
    if (!header.opened) {
      explorerData.loadHeader(headerPath, header);
    } else {
      explorerData.unloadHeader(header);
    }
    header.opened = !header.opened;
  };

  let explorerClick = (name: string) => {
    goto("/main/theorem/" + name);
  };

  let addingSubheader = $state(false);

  let newSubheaderTitle = $state("");

  $effect(() => {
    if (addingSubheader) {
      let input = document.getElementById("subheaderName");
      if (input) {
        input.focus();
      }
    }
  });

  let openAddSubheaderInput = async () => {
    if (!header.opened) {
      await toggleOpen();
    }
    addingSubheader = true;
    newSubheaderTitle = "";
  };

  let addSubheader = async () => {
    if (newSubheaderTitle === "") {
      // TODO: check whether name exists already
      throw Error("Invalid Name");
    }
    addingSubheader = false;
    await invoke("add_header", { title: newSubheaderTitle, insertPath: calcNewPath(header.subHeaders.length) });
    header.subHeaders.push({ title: newSubheaderTitle, opened: true, theoremNames: [], subHeaders: [] });
  };

  let abortAddingSubheader = () => {
    addingSubheader = false;
  };

  let onFocusOutSubheaderTitle = async () => {
    if (addingSubheader) {
      try {
        await addSubheader();
      } catch (error) {
        abortAddingSubheader();
      }
    }
  };

  let onkeyDownSubheaderTitle = (event: KeyboardEvent) => {
    if (event.key == "Enter") {
      try {
        addSubheader();
      } catch (error) {}
    } else if (event.key == "Escape") {
      abortAddingSubheader();
    }
  };

  let openTheoremName: string | null = $derived.by(() => {
    let segments = $page.url.pathname.split("/");
    if (segments.length == 4 && segments[1] == "main" && segments[2] == "theorem") {
      return segments[3];
    }
    return null;
  });
</script>

<div class="relative h-6 hover:bg-gray-200">
  <button class="h-full w-full text-left absolute" onclick={toggleOpen}>
    <div class="h-6 w-6 div float-left">
      {#if header.opened}
        <ChevronDownIcon></ChevronDownIcon>
      {:else}
        <ChevronRightIcon></ChevronRightIcon>
      {/if}
    </div>
    <div class="ml-6 whitespace-nowrap mr-6 overflow-hidden">
      {pathString}
      {header.title}
    </div>
  </button>
  <button aria-label="Add subheader" onclick={openAddSubheaderInput} class="h-4 w-4 absolute bottom-1 end-1">
    <PlusIcon></PlusIcon>
  </button>
</div>
<div class="pl-3">
  {#each header.theoremNames as theoremName}
    <div>
      <button class={"w-full text-left pl-2 " + (theoremName === openTheoremName ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={() => explorerClick(theoremName)}>{theoremName}</button>
    </div>
  {/each}
  {#each header.subHeaders as subHeader, index}
    <ExplorerHeader header={subHeader} headerPath={calcNewPath(index)}></ExplorerHeader>
  {/each}
  {#if addingSubheader}
    <input id="subheaderName" type="text" bind:value={newSubheaderTitle} onfocusout={onFocusOutSubheaderTitle} onkeydown={onkeyDownSubheaderTitle} disabled={!addingSubheader} autocomplete="off" class="disabled:bg-gray-300" />
  {/if}
</div>
