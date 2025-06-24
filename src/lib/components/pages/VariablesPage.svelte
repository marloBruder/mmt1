<script lang="ts">
  import type { VariablesPageData } from "$lib/sharedState/model.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { pageData }: { pageData: VariablesPageData } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">
      {#if pageData.variables.length > 1}
        Variables:
      {:else}
        Variable:
      {/if}
    </h1>
  </div>
  <div class="flex flex-col items-center">
    <table>
      <thead>
        <tr>
          <td class="p-2 border-b border-r border-black">ASCII</td>
          <td class="p-2 border-b border-r border-l border-black">HTML</td>
          <td class="p-2 border-b border-l border-black">Typecode</td>
        </tr>
      </thead>
      <tbody>
        {#each pageData.variables as [variable, typecode]}
          <tr>
            <td class="border-r border-black">{variable.symbol}</td>
            <td class="border-r border-l border-black"><MetamathExpression expression={variable.symbol}></MetamathExpression></td>
            <td class="border-l border-black"><MetamathExpression expression={typecode}></MetamathExpression></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
