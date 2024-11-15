<script lang="ts">
  import editorTabs from "$lib/sharedState/mainData.svelte";
</script>

<div>
  <div class="py-4 flex flex-col items-center">
    <button class="border border-black" onclick={() => editorTabs.addTab()}>Add new theorem</button>
  </div>
  <div>In Progress theorems:</div>
  {#each editorTabs.tabs as tab}
    <div>
      <a href="/main/editor/{tab.id}" class="block {editorTabs.openedTabID == tab.id ? 'bg-gray-300' : 'hover:bg-gray-100'}">{tab.name}</a>
    </div>
  {/each}
</div>
