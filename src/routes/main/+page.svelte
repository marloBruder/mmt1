<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";
  import TabBar from "$lib/components/nav/tabBar/TabBar.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import TabComponent from "$lib/components/tabs/TabComponent.svelte";
  import VerticalDraggableCollapsableSplit from "$lib/components/util/VerticalDraggableCollapsableSplit.svelte";

  let setVerticalDraggableSplitPosition = $state(320);
  let setNavSidebarCollapsed = $state(false);

  let onVerticalDraggableSplitDrag = () => {
    setNavSidebarCollapsed = true;
    setNavSidebarCollapsed = false;
  };

  let onVerticalDraggableSplitCollapse = () => {
    setNavSidebarCollapsed = false;
    setNavSidebarCollapsed = true;
  };

  let onNavSidebarCollapse = () => {
    setVerticalDraggableSplitPosition = 0;
    setVerticalDraggableSplitPosition = 60;
  };

  let onNavSidebarUncollapse = () => {
    setVerticalDraggableSplitPosition = 0;
    setVerticalDraggableSplitPosition = 320;
  };
</script>

<VerticalDraggableCollapsableSplit onDrag={onVerticalDraggableSplitDrag} onCollapse={onVerticalDraggableSplitCollapse} setPosition={setVerticalDraggableSplitPosition}>
  {#snippet first()}
    <div class="custom-height-minus-margin custom-width-minus-margin overflow-hidden custom-bg-color my-2 ml-2 mr-1 rounded-lg">
      <NavSidebar onCollapse={onNavSidebarCollapse} onUncollapse={onNavSidebarUncollapse} setCollapsed={setNavSidebarCollapsed}></NavSidebar>
    </div>
  {/snippet}
  {#snippet second()}
    <div class="custom-height-minus-margin custom-width-minus-margin overflow-hidden my-2 mr-2 ml-1 rounded-lg custom-bg-color">
      <HorizontalSplit>
        {#snippet first()}
          <div class="h-8 w-full overflow-hidden">
            <TabBar></TabBar>
          </div>
        {/snippet}
        {#snippet second()}
          <TabComponent></TabComponent>
        {/snippet}
      </HorizontalSplit>
    </div>
  {/snippet}
</VerticalDraggableCollapsableSplit>

<style>
  .custom-height-minus-margin {
    height: calc(100% - 1rem);
  }

  .custom-width-minus-margin {
    width: calc(100% - 0.75rem);
  }
</style>
