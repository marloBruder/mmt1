<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerHeader from "./ExplorerHeader.svelte";
  import { goto } from "$app/navigation";
  import type { HeaderRepresentation, NameListHeader } from "$lib/sharedState/model.svelte";
  import ChevronDownIcon from "$lib/icons/ChevronDownIcon.svelte";
  import ChevronRightIcon from "$lib/icons/ChevronRightIcon.svelte";
  import PlusIcon from "$lib/icons/PlusIcon.svelte";
  import { page } from "$app/stores";

  let { header, location }: { header: NameListHeader; location: number[] } = $props();

  let locationString = $derived.by(() => {
    let stringRep = "";
    for (let pos of location) {
      stringRep = stringRep + (pos + 1) + ".";
    }
    stringRep = stringRep.slice(0, stringRep.length - 1);
    return stringRep;
  });

  let calcNewLocation = (index: number) => {
    let newLocation = location.slice();
    newLocation.push(index);
    return newLocation;
  };

  let opened = $state(false);

  let toggleOpen = async () => {
    if (!opened) {
      let dataUnknown = await invoke("get_header_local", { headerPath: { path: location } });
      let data = dataUnknown as HeaderRepresentation;
      header.theoremNames = data.theoremNames;
      header.subHeaders = data.subHeaderNames.map((title) => {
        return { title, theoremNames: [], subHeaders: [] };
      });
    } else {
      header.theoremNames = [];
      header.subHeaders = [];
    }
    opened = !opened;
  };

  onMount(() => {
    if (header.subHeaders.length != 0 || header.theoremNames.length != 0) {
      opened = true;
    }
  });

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
    if (!opened) {
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
    await invoke("add_header", { title: newSubheaderTitle, insertPath: { path: calcNewLocation(header.subHeaders.length) } });
    header.subHeaders.push({ title: newSubheaderTitle, theoremNames: [], subHeaders: [] });
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
      {#if opened}
        <ChevronDownIcon></ChevronDownIcon>
      {:else}
        <ChevronRightIcon></ChevronRightIcon>
      {/if}
    </div>
    <div class="ml-6 whitespace-nowrap mr-6 overflow-hidden">
      {locationString}
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
    <ExplorerHeader header={subHeader} location={calcNewLocation(index)}></ExplorerHeader>
  {/each}
  {#if addingSubheader}
    <input id="subheaderName" type="text" bind:value={newSubheaderTitle} onfocusout={onFocusOutSubheaderTitle} onkeydown={onkeyDownSubheaderTitle} disabled={!addingSubheader} autocomplete="off" class="disabled:bg-gray-300" />
  {/if}
</div>
