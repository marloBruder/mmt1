<script lang="ts">
  import type { PageData } from "./$types";
  import editorTabs from "$lib/sharedState/mainData.svelte";

  let { data }: { data: PageData } = $props();

  let tab = $derived(editorTabs.getTabByID(Number.parseInt(data.tabID)));
</script>

{#if tab}
  <p>This is the editor page of WIP theorem: {tab.name}</p>
  <p>Text: {tab.text}</p>
{:else}
  <p>Opened editor tab with id "{data.tabID}" has no data associated with it.</p>
{/if}
