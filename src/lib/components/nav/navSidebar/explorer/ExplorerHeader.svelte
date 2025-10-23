<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ExplorerHeader from "./ExplorerHeader.svelte";
  import type { HeaderPath, NameListHeader } from "$lib/sharedState/model.svelte";
  import ChevronDownIcon from "$lib/icons/arrows/ChevronDownIcon.svelte";
  import ChevronRightIcon from "$lib/icons/arrows/ChevronRightIcon.svelte";
  import PlusIcon from "$lib/icons/PlusIcon.svelte";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import ExplorerCommentButton from "./ExplorerCommentButton.svelte";
  import ExplorerFloatingHypothesisButton from "./ExplorerFloatingHypothesisButton.svelte";
  import ExplorerVariableButton from "./ExplorerVariableButton.svelte";
  import ExplorerConstantButton from "./ExplorerConstantButton.svelte";
  import ExplorerTheoremButton from "./ExplorerTheoremButton.svelte";
  import ContextMenuElement from "$lib/components/util/contextMenu/ContextMenuElement.svelte";
  import ContextMenuButton from "$lib/components/util/contextMenu/ContextMenuButton.svelte";
  import { TheoremExplorerTab } from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";

  let { header, headerPath }: { header: NameListHeader; headerPath: HeaderPath } = $props();

  let pathString = $derived(util.headerPathToStringRep(headerPath));

  let calcNewPath = (index: number): HeaderPath => {
    let newPath = { path: headerPath.path.slice() };
    newPath.path.push(index);
    return newPath;
  };

  let toggleOpen = async () => {
    if (header.content === null) {
      explorerData.loadHeader(headerPath, header);
    } else {
      commentNum = 0;
      explorerData.unloadHeader(header);
    }
  };

  let addingSubheader = $state(false);

  let newSubheaderTitle = $state("");

  $effect(() => {
    if (addingSubheader) {
      document.getElementById("subheaderName")!.focus();
    }
  });

  let openAddSubheaderInput = async () => {
    if (header.content === null) {
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
    if (header.content) {
      await invoke("add_header", { title: newSubheaderTitle, insertPath: calcNewPath(header.content!.subheaders.length) });
      header.content.subheaders.push({ title: newSubheaderTitle, content: null });
    }
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

  let commentNum = 0;

  let newCommentNum = () => {
    commentNum++;
    return commentNum - 1;
  };

  let openInNewTheoremExplorer = async () => {
    let pageNum = (await invoke("get_theorem_list_page_of_header", { headerPath })) as number;

    tabManager.openTab(new TheoremExplorerTab(pageNum, "header-list-entry-id-" + util.headerPathToStringRep(headerPath)), true);
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div class="h-6 custom-bg-hover-color">
      <button class="h-full w-full text-left flex flex-row" onclick={toggleOpen}>
        <div class="h-6 w-6">
          {#if header.content !== null}
            <ChevronDownIcon></ChevronDownIcon>
          {:else}
            <ChevronRightIcon></ChevronRightIcon>
          {/if}
        </div>
        <div class="whitespace-nowrap overflow-hidden">
          {pathString}
          {header.title}
        </div>
      </button>
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <ContextMenuButton onclick={openInNewTheoremExplorer}>Open In New Theorem Explorer</ContextMenuButton>
  {/snippet}
</ContextMenuElement>
{#if header.content !== null}
  <div class="pl-3">
    {#each header.content.contentTitles as contentTitle}
      {#if contentTitle.contentType === "CommentStatement"}
        <ExplorerCommentButton {headerPath} commentNum={newCommentNum()}></ExplorerCommentButton>
      {:else if contentTitle.contentType === "ConstantStatement"}
        <ExplorerConstantButton constants={contentTitle.title}></ExplorerConstantButton>
      {:else if contentTitle.contentType === "VariableStatement"}
        <ExplorerVariableButton variables={contentTitle.title}></ExplorerVariableButton>
      {:else if contentTitle.contentType === "FloatingHypothesisStatement"}
        <ExplorerFloatingHypothesisButton label={contentTitle.title}></ExplorerFloatingHypothesisButton>
      {:else if contentTitle.contentType === "TheoremStatement"}
        <ExplorerTheoremButton label={contentTitle.title}></ExplorerTheoremButton>
      {/if}
    {/each}
    {#each header.content.subheaders as subHeader, index}
      <ExplorerHeader header={subHeader} headerPath={calcNewPath(index)}></ExplorerHeader>
    {/each}
    {#if addingSubheader}
      <input id="subheaderName" type="text" bind:value={newSubheaderTitle} onfocusout={onFocusOutSubheaderTitle} onkeydown={onkeyDownSubheaderTitle} disabled={!addingSubheader} autocomplete="off" class="disabled:bg-gray-300" />
    {/if}
  </div>
{/if}
