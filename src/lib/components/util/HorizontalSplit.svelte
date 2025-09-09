<script lang="ts">
  import type { Snippet } from "svelte";

  let { first, second, secondFixed = false }: { first: Snippet; second: Snippet; secondFixed?: boolean } = $props();
</script>

<div class="h-full flex flex-col">
  <div class={!secondFixed ? " flex-initial " : " flex-auto overflow-hidden"}>
    {@render first()}
  </div>
  <div class={!secondFixed ? " flex-auto overflow-hidden" : " flex-initial "}>
    {@render second()}
  </div>
</div>
