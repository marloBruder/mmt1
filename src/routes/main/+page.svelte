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
  <div class="h-full w-80 fixed">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80 border-l border-gray-400 custom-grid-layout">
    <div class="h-8">
      <TabBar></TabBar>
    </div>
    <div class="overflow-auto">
      {#if openedTab instanceof TheoremTab}
        <TheoremTabComponent tab={openedTab}></TheoremTabComponent>
      {:else if openedTab instanceof EditorTab}
        <EditorTabComponent tab={openedTab}></EditorTabComponent>
      {:else if openedTab instanceof SettingsTab}
        <SettingsTabComponent tab={openedTab}></SettingsTabComponent>
      {:else}
        <EmptyTabComponent></EmptyTabComponent>
      {/if}
    </div>
  </div>
</div>

<style>
  .custom-grid-layout {
    display: grid;
    grid-template-rows: 2rem;
  }
</style>
