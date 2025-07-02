<script lang="ts">
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import { type Snippet } from "svelte";

  let { children, theme = "os-custom-scrollbar-theme" }: { children: Snippet; theme?: string } = $props();
</script>

<OverlayScrollbarsComponent class="h-full w-full" options={{ scrollbars: { autoHide: "leave", clickScroll: "instant", theme } }}>
  {@render children()}
</OverlayScrollbarsComponent>
