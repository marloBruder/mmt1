<script lang="ts">
  import { EditorTab } from "$lib/components/tabs/EditorTabComponent.svelte";
  import ContextMenuElement from "$lib/components/util/ContextMenuElement.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";

  let { folderPath, fileName }: { folderPath: string; fileName: string } = $props();

  let explorerClick = () => {
    tabManager.openTab(new EditorTab(folderPath + fileName));
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(new EditorTab(folderPath + fileName));
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div>
      <button class="w-full text-left pl-2 custom-bg-hover-color" onclick={explorerClick} ondblclick={explorerDblClick}>{fileName}</button>
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <div><button class="w-full px-2 text-left hover:bg-purple-500">Open</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500">Copy Path</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500">Copy Relative Path</button></div>
    <div class="py-1"><hr /></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500">Rename</button></div>
    <div><button class="w-full px-2 text-left hover:bg-purple-500">Delete</button></div>
  {/snippet}
</ContextMenuElement>
