<script lang="ts">
  import TitleBar from "$lib/components/titleBar/TitleBar.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import { setupTheoremNumberStyleSheet } from "$lib/components/util/TheoremNumber.svelte";
  import { setupShortcuts } from "$lib/sharedState/shortcuts.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  onMount(() => {
    invoke("setup_main_window");
    setupTheoremNumberStyleSheet();
    setupShortcuts();
  });
</script>

<div class="h-screen w-screen fixed custom-bg-bg-color text-gray-300">
  <HorizontalSplit>
    {#snippet first()}
      <div class="h-8 w-full overflow-hidden">
        <TitleBar></TitleBar>
      </div>
    {/snippet}
    {#snippet second()}
      <slot />
    {/snippet}
  </HorizontalSplit>
</div>
