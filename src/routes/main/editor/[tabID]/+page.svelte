<script lang="ts">
  import type { PageData } from "./$types";
  import editorTabs from "$lib/sharedState/mainData.svelte";

  let { data }: { data: PageData } = $props();

  let tab = $derived(editorTabs.getTabByID(data.tabID));

  let oldName: string = $state("");

  let nameDisabled: boolean = $state(true);

  let editName = () => {
    oldName = tab ? tab.name : "";
    nameDisabled = false;
  };

  let saveName = () => {
    if (tab && editorTabs.nameExists(tab.id, tab.name)) {
      return;
    }
    nameDisabled = true;
  };

  let abortNameSave = () => {
    nameDisabled = true;
    if (tab) {
      tab.name = oldName;
    }
  };

  $effect(() => {
    editorTabs.openedTabID = data.tabID;
  });
</script>

{#if tab}
  <div class="m-2">
    <div class="mb-2">
      <label for="tabName">Theorem name:</label>
      <input id="tabName" type="text" bind:value={tab.name} disabled={nameDisabled} class="disabled:bg-gray-300" />
    </div>
    <button onclick={editName} disabled={!nameDisabled} class="border border-black rounded px-1 disabled:bg-gray-300">Edit name</button>
    <button onclick={saveName} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Save name</button>
    <button onclick={abortNameSave} disabled={nameDisabled} class="ml-4 border border-black rounded px-1 disabled:bg-gray-300">Abort edit</button>
  </div>
  <div>
    <textarea bind:value={tab.text} class="w-full resize-none h-96"></textarea>
  </div>
{:else}
  <p>Opened editor tab with id "{data.tabID}" has no data associated with it.</p>
{/if}
