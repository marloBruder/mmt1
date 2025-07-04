<script lang="ts">
  import NavSidebar from "$lib/components/nav/navSidebar/NavSidebar.svelte";
  import TabBar from "$lib/components/nav/tabBar/TabBar.svelte";
  import TitleBar from "$lib/components/titleBar/TitleBar.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import VerticalSplit from "$lib/components/util/VerticalSplit.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TabComponent from "$lib/components/tabs/TabComponent.svelte";

  onMount(() => {
    invoke("setup_main_window");
  });
</script>

<div class="h-screen w-screen fixed custom-bg-bg-color text-gray-300">
  <HorizontalSplit>
    {#snippet first()}
      <div class="h-8 w-full overflow-hidden">
        <TitleBar></TitleBar>
      </div>
    {/snippet}
    {#snippet second()}
      <VerticalSplit>
        {#snippet first()}
          <div class="custom-height-minus-margin w-80 overflow-hidden custom-bg-color my-2 ml-2 mr-1 rounded-lg">
            <NavSidebar></NavSidebar>
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
      </VerticalSplit>
    {/snippet}
  </HorizontalSplit>
</div>

<style>
  .custom-height-minus-margin {
    height: calc(100% - 1rem);
  }

  .custom-width-minus-margin {
    width: calc(100% - 0.75rem);
  }
</style>
