<script lang="ts">
  import NavSidebarButtons from "./NavSidebarButtons.svelte";
  import NavSidebarExplorer from "./NavSidebarExplorer.svelte";
  import NavSidebarSearch from "./NavSidebarSearch.svelte";
  import NavSidebarEditor from "./NavSidebarEditor.svelte";
  import SearchIcon from "$lib/icons/navSidebar/SearchIcon.svelte";
  import FileIcon from "$lib/icons/navSidebar/FileIcon.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import AlephZeroIcon from "$lib/icons/navSidebar/AlephZeroIcon.svelte";

  let tabInfo = [
    {
      title: "Explorer",
      component: NavSidebarExplorer,
      icon: AlephZeroIcon,
    },
    {
      title: "Search",
      component: NavSidebarSearch,
      icon: SearchIcon,
    },
    {
      title: "Editor",
      component: NavSidebarEditor,
      icon: FileIcon,
    },
  ];

  let activeTab = $state(0);
  let TabComponent = $derived(tabInfo[activeTab].component);

  let onNavButtonClick = (tabIndex: number) => {
    activeTab = tabIndex;
  };
</script>

<div class="h-full">
  <div class="h-full w-12 float-left">
    <NavSidebarButtons {activeTab} {tabInfo} onClick={onNavButtonClick} />
  </div>
  <div class="h-full ml-12 border-l-2 custom-border-bg-color overflow-x-hidden">
    <ScrollableContainer>
      <TabComponent></TabComponent>
    </ScrollableContainer>
  </div>
</div>
