<script lang="ts">
  import NavSidebar from "$lib/navigation/NavSidebar.svelte";

	let { children } = $props();
</script>

<div class="h-screen w-screen">
  <div class="h-full w-80 fixed top-0 left-0">
    <NavSidebar></NavSidebar>
  </div>
  <div class="h-full ml-80">
    {@render children()}
  </div>
</div>