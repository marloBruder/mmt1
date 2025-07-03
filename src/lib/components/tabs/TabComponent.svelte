<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ScrollableContainer from "../util/ScrollableContainer.svelte";
  import EmptyTabComponent from "./EmptyTabComponent.svelte";

  let openTab = $derived(tabManager.getOpenTab());

  $effect(() => {
    if (openTab) {
      let tabContainer = document.getElementById("tabContainer");
      if (tabContainer !== null) {
        tabContainer.scrollTop = openTab.scrollTop;
      }
    }
  });

  let onscrollTab = (e: UIEvent) => {
    if (openTab) {
      openTab.scrollTop = (e.target as HTMLElement).scrollTop;
    }
  };

  let verticalSplit = $derived(tabManager.splitTabState === "splitVertical" && openTab != null && openTab.splitComponent != null);
  let horizontalSplit = $derived(tabManager.splitTabState === "splitHorizontal" && openTab != null && openTab.splitComponent != null);
</script>

{#if openTab != null}
  <div id="tabContainer" class="h-full w-full flex {verticalSplit ? 'flex-row' : 'flex-col'}" onscroll={onscrollTab}>
    <div class="{horizontalSplit ? 'h-1/2' : 'h-full'} {verticalSplit ? 'w-1/2' : 'w-full'}">
      <ScrollableContainer>
        <openTab.component tab={openTab}></openTab.component>
      </ScrollableContainer>
    </div>
    {#if (verticalSplit || horizontalSplit) && openTab.splitComponent !== null}
      <div class="{horizontalSplit ? 'h-1/2' : 'h-full'} {verticalSplit ? 'w-1/2' : 'w-full'}">
        <ScrollableContainer>
          <openTab.splitComponent pageData={openTab.splitViewPageData}></openTab.splitComponent>
        </ScrollableContainer>
      </div>
    {/if}
  </div>
{:else}
  <EmptyTabComponent></EmptyTabComponent>
{/if}
