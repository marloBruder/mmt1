<script lang="ts">
  import { htmlData } from "$lib/sharedState/htmlData.svelte";

  let { expression }: { expression: string } = $props();

  let symbols = $derived(expression.split(" "));

  let htmlReps = $derived(
    symbols.map((symbol) => {
      if (symbol.includes("$")) {
        let htmlOpt = htmlData.getHtml(symbol.split("$")[0]);
        if (htmlOpt === undefined) {
          return undefined;
        }
        let [htmlRep, typecodeNum] = htmlOpt;
        return [htmlRep + "$" + symbol.split("$")[1], typecodeNum];
      } else {
        return htmlData.getHtml(symbol);
      }
    })
  );
</script>

{#each htmlReps as htmlRep, i}
  {#if i !== 0}
    {" "}
  {/if}
  {#if htmlRep}
    <span class={"math " + (htmlRep[1] !== 0 ? " custom-variable-color-" + htmlRep[1] : "")}>
      <span>{@html htmlRep[0]}</span>
    </span>
  {:else}
    {symbols[i]}
  {/if}
{/each}
