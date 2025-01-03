<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";
  import TabBar from "$lib/components/nav/TabBar.svelte";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="h-screen w-screen custom-grid-layout fixed">
  <div class="sideBar border-r border-gray-400 overflow-hidden">
    <NavSidebar></NavSidebar>
  </div>
  <div class="tabBar overflow-hidden">
    <TabBar></TabBar>
  </div>
  <div class="tab overflow-auto">
    {@render children()}
  </div>
  <!-- <div class="h-full ml-80 border-l border-gray-400">
  </div> -->
</div>

<style>
  .custom-grid-layout {
    display: grid;
    grid-template-areas:
      "sideBar tabBar"
      "sideBar tab";
    grid-template-columns: 20rem auto;
    grid-template-rows: 2rem auto;
  }

  .sideBar {
    grid-area: sideBar;
  }

  .tabBar {
    grid-area: tabBar;
  }

  .tab {
    grid-area: tab;
  }
</style>
