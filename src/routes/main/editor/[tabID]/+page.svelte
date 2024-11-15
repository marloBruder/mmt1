<script lang="ts">
  import type { PageData } from "./$types";
  import editorTabs from "$lib/sharedState/mainData.svelte";

  let { data }: { data: PageData } = $props();

  let tab = $derived(editorTabs.getTabByID(data.tabID));

  $effect(() => {
    editorTabs.openedTabID = data.tabID;
  });
</script>

{#if tab}
  <div>
    <label for="tabName">Theorem name:</label>
    <input id="tabName" type="text" bind:value={tab.name} />
  </div>
  <div>
    <textarea bind:value={tab.text} class="w-full resize-none h-96"></textarea>
  </div>
{:else}
  <p>Opened editor tab with id "{data.tabID}" has no data associated with it.</p>
{/if}
