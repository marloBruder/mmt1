<script lang="ts">
  import { htmlData } from "$lib/sharedState/htmlData.svelte";

  let { expression }: { expression: string } = $props();
</script>

{#each expression.split(" ") as symbol}
  {@html htmlData.getHtml(symbol) || symbol}{" "}
{/each}
<!-- {expression} -->
