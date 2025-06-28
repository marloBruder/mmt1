<script lang="ts">
  import type { Snippet } from "svelte";

  let { first, second }: { first: Snippet; second: Snippet } = $props();
</script>

<div class="h-full flex flex-row">
  <div class="flex-initial">
    {@render first()}
  </div>
  <div class="flex-auto overflow-hidden">
    {@render second()}
  </div>
</div>
