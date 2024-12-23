<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";
  import TabBar from "$lib/components/nav/TabBar.svelte";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
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
      {@render children()}
    </div>
  </div>
</div>

<style>
  .custom-grid-layout {
    display: grid;
    grid-template-rows: 2rem;
  }
</style>
