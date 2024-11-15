<script>
  import editorTabs from "$lib/sharedState/mainData.svelte";
</script>

<div>
  <div class="py-4 flex flex-col items-center">
    <button class="border border-black" onclick={() => editorTabs.addTab()}>Add new theorem</button>
  </div>
  <div>In Progress theorems:</div>
  {#each editorTabs.tabs as tab}
    <div class="hover:bg-gray-100">
      <a href="/main/editor/{tab.id}" class="block">{tab.name}</a>
    </div>
  {/each}
</div>
