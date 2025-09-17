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

<ContextMenuElement context="fileButton">
  <div>
    <button class="w-full text-left pl-2 custom-bg-hover-color" onclick={explorerClick} ondblclick={explorerDblClick}>{fileName}</button>
  </div>
</ContextMenuElement>
