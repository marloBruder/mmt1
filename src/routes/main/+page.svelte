<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";

  import { tabManager } from "$lib/sharedState/tabData.svelte";
  import { EditorTabClass, TheoremTabClass } from "$lib/sharedState/tabData.svelte";
  import EmptyTab from "$lib/components/tabs/EmptyTab.svelte";
  import TheoremTab from "$lib/components/tabs/TheoremTab.svelte";
  import EditorTab from "$lib/components/tabs/EditorTab.svelte";
  import TabBar from "$lib/components/nav/TabBar.svelte";

  let openedTab = $derived(tabManager.getOpenedTab());
</script>

<div class="h-screen w-screen">
  <div class="h-full w-80 fixed top-0 left-0">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80 border-l border-gray-400 overflow-y-scroll overflow-x-hidden">
    <TabBar></TabBar>
    {#if openedTab instanceof TheoremTabClass}
      <TheoremTab theoremName={openedTab.theoremName}></TheoremTab>
    {:else if openedTab instanceof EditorTabClass}
      <EditorTab localID={openedTab.localID}></EditorTab>
    {:else}
      <EmptyTab></EmptyTab>
    {/if}
  </div>
</div>
