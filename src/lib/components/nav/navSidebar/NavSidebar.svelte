<script lang="ts">
  import NavSidebarButtons from "./NavSidebarButtons.svelte";
  import NavSidebarExplorer from "./NavSidebarExplorer.svelte";
  import NavSidebarSearch from "./NavSidebarSearch.svelte";
  import NavSidebarEditor from "./NavSidebarEditor.svelte";
  import BookIcon from "$lib/icons/navSidebar/BookIcon.svelte";
  import SearchIcon from "$lib/icons/navSidebar/SearchIcon.svelte";
  import FileIcon from "$lib/icons/navSidebar/FileIcon.svelte";

  let tabInfo = [
    {
      title: "Explorer",
      component: NavSidebarExplorer,
      icon: BookIcon,
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
  <div class="h-full ml-12 border-l border-gray-400 overflow-y-auto overflow-x-hidden">
    <TabComponent></TabComponent>
  </div>
</div>
