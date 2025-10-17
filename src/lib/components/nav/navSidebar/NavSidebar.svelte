<script lang="ts" module>
  export interface NavSidebarTabInfo {
    title: string;
    scrollTop: number;
    component: Component;
    icon: Component;
  }
</script>

<script lang="ts">
  import NavSidebarButtons from "./NavSidebarButtons.svelte";
  import NavSidebarExplorer from "./NavSidebarExplorer.svelte";
  import NavSidebarSearch from "./NavSidebarSearch.svelte";
  import NavSidebarEditor from "./NavSidebarEditor.svelte";
  import SearchIcon from "$lib/icons/navSidebar/SearchIcon.svelte";
  import FileIcon from "$lib/icons/navSidebar/FileIcon.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import AlephZeroIcon from "$lib/icons/navSidebar/AlephZeroIcon.svelte";
  import type { Component } from "svelte";

  let { onCollapse, onUncollapse, setCollapsed = false }: { onCollapse: () => void; onUncollapse: () => void; setCollapsed?: boolean } = $props();

  let tabInfo: NavSidebarTabInfo[] = [
    {
      title: "Explorer",
      scrollTop: 0,
      component: NavSidebarExplorer,
      icon: AlephZeroIcon,
    },
    {
      title: "Search",
      scrollTop: 0,
      component: NavSidebarSearch,
      icon: SearchIcon,
    },
    {
      title: "Editor",
      scrollTop: 0,
      component: NavSidebarEditor,
      icon: FileIcon,
    },
  ];

  let activeTab = $state(0);
  let isCollapsed = $state(false);
  let TabComponent = $derived(tabInfo[activeTab].component);

  let setScrollTop = $state(0);

  let onNavButtonClick = (tabIndex: number) => {
    if (!isCollapsed && activeTab == tabIndex) {
      isCollapsed = true;
      onCollapse();
    } else {
      if (isCollapsed) {
        onUncollapse();
        isCollapsed = false;
      }
      activeTab = tabIndex;
      // make sure setScrollTop changes
      setScrollTop = -1;
      setScrollTop = tabInfo[tabIndex].scrollTop;
    }
  };

  let onTabComponentScroll = (newScrollTop: number) => {
    tabInfo[activeTab].scrollTop = newScrollTop;
  };

  $effect(() => {
    isCollapsed = setCollapsed;
  });
</script>

<div class="h-full">
  <div class="h-full w-12 float-left">
    <NavSidebarButtons {activeTab} {isCollapsed} {tabInfo} onClick={onNavButtonClick} />
  </div>
  {#if !isCollapsed}
    <div class="h-full ml-12 border-l-2 custom-border-bg-color overflow-x-hidden">
      <ScrollableContainer scrollTop={setScrollTop} onscroll={onTabComponentScroll}>
        <TabComponent></TabComponent>
      </ScrollableContainer>
    </div>
  {/if}
</div>
