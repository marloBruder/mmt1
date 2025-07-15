<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import "overlayscrollbars/overlayscrollbars.css";
  import { invoke } from "@tauri-apps/api/core";
  import { setupShortcuts } from "$lib/sharedState/shortcuts.svelte";

  onMount(() => {
    invoke("setup_main_window");
    setupShortcuts();
  });
</script>

<slot />
