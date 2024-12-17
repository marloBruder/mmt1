<script lang="ts">
  import NavSidebar from "$lib/navSidebar/NavSidebar.svelte";

  import tabManager, { Tab } from "$lib/sharedState/tabData.svelte";
  import { EditorTabClass, TheoremTabClass } from "$lib/sharedState/tabData.svelte";
  import EmptyTab from "$lib/tabs/EmptyTab.svelte";
  import TheoremTab from "$lib/tabs/TheoremTab.svelte";
  import EditorTab from "$lib/tabs/EditorTab.svelte";

  let openedTab = $derived(tabManager.getOpenedTab());
</script>

<div class="h-screen w-screen">
  <div class="h-full w-80 fixed top-0 left-0">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80 border-l border-gray-400 overflow-y-scroll overflow-x-hidden">
    <div class="h-8 border-b border-gray-400">
      {#each tabManager.tabs as tab, index}
        <button
          onclick={() => {
            tabManager.openTab(index);
          }}
          class="h-full px-2 border-r border-gray-400">{tab.name()}</button
        >
      {/each}
    </div>
    {#if openedTab instanceof TheoremTabClass}
      <TheoremTab theoremName={openedTab.theoremName}></TheoremTab>
    {:else if openedTab instanceof EditorTabClass}
      <EditorTab localID={openedTab.localID}></EditorTab>
    {:else}
      <EmptyTab></EmptyTab>
    {/if}
  </div>
</div>
