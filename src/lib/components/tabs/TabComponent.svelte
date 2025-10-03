<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import ScrollableContainer from "../util/ScrollableContainer.svelte";
  import EmptyTabComponent from "./EmptyTabComponent.svelte";

  let openTab = $derived(tabManager.getOpenTab());
  let scrollTop = $state(0);
  let splitViewScrollTop = $state(0);

  // Triggers only when openTab changes
  $effect(() => {
    if (openTab) {
      // make sure scrollTop and splitViewScrollTop change
      scrollTop = -1;
      scrollTop = openTab.scrollTop;
      splitViewScrollTop = -1;
      splitViewScrollTop = openTab.splitViewScrollTop;
    }
  });

  let onscrollTab = (newScrollTop: number) => {
    if (openTab) {
      openTab.scrollTop = newScrollTop;
    }
  };

  let onscrollSplitView = (newScrollTop: number) => {
    if (openTab) {
      openTab.splitViewScrollTop = newScrollTop;
    }
  };

  let verticalSplit = $derived(tabManager.splitTabState === "splitVertical" && openTab != null && openTab.splitComponent != null);
  let horizontalSplit = $derived(tabManager.splitTabState === "splitHorizontal" && openTab != null && openTab.splitComponent != null);
</script>

{#if openTab != null}
  <div class="h-full w-full flex {verticalSplit ? 'flex-row' : 'flex-col'}">
    <div class="{horizontalSplit ? 'h-1/2' : 'h-full'} {verticalSplit ? 'w-1/2' : 'w-full'}">
      <ScrollableContainer onscroll={onscrollTab} {scrollTop}>
        <openTab.component tab={openTab}></openTab.component>
      </ScrollableContainer>
    </div>
    {#if (verticalSplit || horizontalSplit) && openTab.splitComponent !== null}
      <div class="{horizontalSplit ? 'h-1/2' : 'h-full'} {verticalSplit ? 'w-1/2' : 'w-full'}">
        <ScrollableContainer onscroll={onscrollSplitView} scrollTop={splitViewScrollTop}>
          <openTab.splitComponent pageData={openTab.splitViewPageData}></openTab.splitComponent>
        </ScrollableContainer>
      </div>
    {/if}
  </div>
{:else}
  <EmptyTabComponent></EmptyTabComponent>
{/if}
