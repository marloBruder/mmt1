<script lang="ts">
  import type { ConstantsPageData } from "$lib/sharedState/model.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { pageData }: { pageData: ConstantsPageData } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">
      {#if pageData.constants.length > 1}
        Constants:
      {:else}
        Constant:
      {/if}
    </h1>
  </div>
  <div class="flex flex-col items-center">
    <table>
      <thead>
        <tr>
          <td class="p-2 border-b border-r border-gray-300">ASCII</td>
          <td class="p-2 border-b border-l border-gray-300">HTML</td>
        </tr>
      </thead>
      <tbody>
        {#each pageData.constants as constant}
          <tr>
            <td class="border-r border-gray-300">{constant.symbol}</td>
            <td class="border-l border-gray-300"><MetamathExpression expression={constant.symbol}></MetamathExpression></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
