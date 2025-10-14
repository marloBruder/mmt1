<script lang="ts">
  import { goto } from "$app/navigation";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
  import ScrollableContainer from "$lib/components/util/ScrollableContainer.svelte";
  import VerticalDraggableSplit from "$lib/components/util/VerticalDraggableSplit.svelte";
  import VerticalSplit from "$lib/components/util/VerticalSplit.svelte";
  import monaco from "$lib/monaco/monaco";
  import { onMount } from "svelte";

  let onCancelClick = () => {
    goto("/main");
  };

  onMount(() => {
    let editorContainer = document.getElementById("editor-div")!;
    let editor = monaco.editor.create(editorContainer, {
      automaticLayout: true,
      fixedOverflowWidgets: true,
      theme: "mmp-theme",
      minimap: { enabled: false },
      stickyScroll: { enabled: false },
      wordBasedSuggestions: "off",
    });
  });
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <HorizontalSplit>
    {#snippet first()}
      <div class="w-full text-left py-2">
        <button class="pl-4" onclick={onCancelClick}>{"< Cancel"}</button>
      </div>
      <div class="text-center text-3xl pt-2 pb-10">Add to Database</div>
    {/snippet}
    {#snippet second()}
      <div class="h-full">
        <VerticalDraggableSplit startPercent={0.8}>
          {#snippet first()}
            <div class="px-2 w-full h-full">
              <div id="editor-div" class="w-full h-full"></div>
            </div>
          {/snippet}
          {#snippet second()}
            <div class="ml-2 h-full w-full border-t border-l rounded-tl-lg">
              <ScrollableContainer>
                <div></div>
              </ScrollableContainer>
            </div>
          {/snippet}
        </VerticalDraggableSplit>
      </div>
    {/snippet}
  </HorizontalSplit>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
