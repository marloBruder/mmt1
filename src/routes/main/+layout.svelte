<script lang="ts">
  import NavSidebar from "$lib/navSidebar/NavSidebar.svelte";
  import { writable } from "svelte/store";

  let { children } = $props();
</script>

<div class="h-screen w-screen">
  <div class="h-full w-80 fixed top-0 left-0">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80 border-l border-gray-400 overflow-y-scroll overflow-x-hidden">
    {@render children()}
  </div>
</div>
