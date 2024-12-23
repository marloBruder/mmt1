<script lang="ts">
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";
  import { onMount } from "svelte";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let theorem = $derived(data.tab.theorem);
</script>

<div class="text-center py-4">
  <h1 class="text-3xl">
    {#if theorem.proof}Theorem
    {:else}Axiom
    {/if}
    {theorem.name}
  </h1>
</div>
<div class="text-center">
  {#if theorem.hypotheses.length != 0}
    <div class="pb-4">
      <h2>Hyptheses:</h2>
      <div class="flex justify-center">
        <table>
          <tbody>
            {#each theorem.hypotheses as hypothesis}
              <tr>
                <td>{hypothesis.label}: </td>
                <td>{hypothesis.hypothesis}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
  <div class="pb-4">
    <h2>Assertion:</h2>
    <p>{theorem.assertion}</p>
  </div>
  {#if theorem.disjoints.length != 0}
    <div class="pb-4">
      <h2>Disjoints:</h2>
      {#each theorem.disjoints as disjoint}
        <p>{disjoint}</p>
      {/each}
    </div>
  {/if}
  <div>
    <h2>Description:</h2>
    <p>{theorem.description}</p>
  </div>
</div>
