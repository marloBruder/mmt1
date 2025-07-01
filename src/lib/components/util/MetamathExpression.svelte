<script lang="ts">
  import { htmlData } from "$lib/sharedState/htmlData.svelte";

  let { expression }: { expression: string } = $props();

  let symbols = $derived(expression.split(" "));

  let htmlReps = $derived(symbols.map((symbol) => htmlData.getHtml(symbol)));
</script>

{#each htmlReps as htmlRep, i}
  {#if i !== 0}
    {" "}
  {/if}
  {#if htmlRep}
    <span class={"math " + (htmlRep[1] !== 0 ? " custom-variable-color-" + htmlRep[1] : "")}>
      {@html htmlRep[0]}{" "}
    </span>
  {:else}
    {symbols[i]}
  {/if}
{/each}
