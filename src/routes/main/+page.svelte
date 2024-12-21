<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";

  import { SettingsTab, tabManager } from "$lib/sharedState/tabData.svelte";
  import { EditorTab, TheoremTab } from "$lib/sharedState/tabData.svelte";
  import EmptyTabComponent from "$lib/components/tabs/EmptyTabComponent.svelte";
  import EditorTabComponent from "$lib/components/tabs/EditorTabComponent.svelte";
  import TheoremTabComponent from "$lib/components/tabs/TheoremTabComponent.svelte";
  import TabBar from "$lib/components/nav/TabBar.svelte";
  import SettingsTabComponent from "$lib/components/tabs/SettingsTabComponent/SettingsTabComponent.svelte";

  let openedTab = $derived(tabManager.getOpenedTab());
</script>

<div class="h-screen w-screen">
  <div class="h-full w-80 fixed top-0 left-0">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80 border-l border-gray-400 overflow-y-scroll overflow-x-hidden">
    <TabBar></TabBar>
    {#if openedTab instanceof TheoremTab}
      <TheoremTabComponent theoremName={openedTab.theoremName}></TheoremTabComponent>
    {:else if openedTab instanceof EditorTab}
      <EditorTabComponent localID={openedTab.localID}></EditorTabComponent>
    {:else if openedTab instanceof SettingsTab}
      <SettingsTabComponent></SettingsTabComponent>
    {:else}
      <EmptyTabComponent></EmptyTabComponent>
    {/if}
  </div>
</div>
