<script lang="ts">
  import ContextMenuButton from "$lib/components/util/contextMenu/ContextMenuButton.svelte";
  import ContextMenuElement from "$lib/components/util/contextMenu/ContextMenuElement.svelte";
  import type { Tab } from "$lib/sharedState/tab.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import type { Snippet } from "svelte";

  let { children, newTab, openInNewTheoremExplorer }: { children: Snippet; newTab: () => Tab; openInNewTheoremExplorer: () => Promise<void> } = $props();

  let isOpenTab = $derived(tabManager.isSameTabOpen(newTab()));

  let explorerClick = (permanent: boolean) => {
    tabManager.openTab(newTab(), permanent);
  };

  let explorerDblClick = () => {
    tabManager.makeSameTempTabPermanent(newTab());
  };
</script>

<ContextMenuElement>
  {#snippet element()}
    <div>
      <button class={"w-full text-left pl-2 " + (isOpenTab ? " custom-bg-active-color " : " custom-bg-hover-color ")} onclick={() => explorerClick(false)} ondblclick={explorerDblClick}>
        {@render children()}
      </button>
    </div>
  {/snippet}
  {#snippet contextMenu()}
    <ContextMenuButton onclick={() => explorerClick(true)}>Open</ContextMenuButton>
    <ContextMenuButton onclick={openInNewTheoremExplorer}>Open In New Theorem Explorer</ContextMenuButton>
  {/snippet}
</ContextMenuElement>
