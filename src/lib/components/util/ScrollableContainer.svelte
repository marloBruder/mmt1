<script lang="ts">
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import { type Snippet } from "svelte";

  let { children, theme = "os-custom-scrollbar-theme", horizontalScroll = false }: { children: Snippet; theme?: string; horizontalScroll?: boolean } = $props();

  let osRef: OverlayScrollbarsComponent | undefined = $state();
  let scrollTarget = 0;

  let onwheel = (event: WheelEvent) => {
    if (horizontalScroll) {
      let osInstance = osRef?.osInstance();

      if (!osInstance) {
        return;
      }
      const { scrollOffsetElement } = osInstance.elements();

      scrollTarget += event.deltaY;
      scrollTarget = Math.max(0, Math.min(scrollTarget, scrollOffsetElement.scrollWidth - scrollOffsetElement.clientWidth));

      scrollOffsetElement.scrollTo({ behavior: "smooth", left: scrollTarget });
    }
  };
</script>

<OverlayScrollbarsComponent bind:this={osRef} {onwheel} class="h-full w-full" options={{ scrollbars: { autoHide: "leave", clickScroll: "instant", theme } }}>
  {@render children()}
</OverlayScrollbarsComponent>
