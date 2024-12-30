<script lang="ts">
  import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";
  import { onMount } from "svelte";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let pageData = $derived(data.tab.pageData);
  let theorem = $derived(pageData.theorem);
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
  <div class="pb-4">
    <h2>Description:</h2>
    <p>{theorem.description}</p>
  </div>
  {#if theorem.proof != null}
    <div class="pb-4">
      <h2>Raw Proof:</h2>
      <p>{theorem.proof}</p>
    </div>
    <div>
      <h2>Proof</h2>
      <table class="mx-auto border text-left border-collapse">
        <thead>
          <tr>
            <th class="border border-gray-600 py-1 px-2">Step</th>
            <th class="border border-gray-600 py-1 px-2">Hyp</th>
            <th class="border border-gray-600 py-1 px-2">Ref</th>
            <th class="border border-gray-600 py-1 px-2">Statement</th>
          </tr>
        </thead>
        <tbody>
          {#each pageData.proofLines as proofLine, index}
            <tr>
              <td class="border border-gray-600 py-1 px-2">{index + 1}</td>
              <td class="border border-gray-600 py-1 px-2">
                {#each proofLine.hypotheses as hypothesis, index}
                  {hypothesis + (index != proofLine.hypotheses.length - 1 ? ", " : "")}
                {/each}
              </td>
              <td class="border border-gray-600 py-1 px-2"> <a href={"/main/theorem/" + proofLine.reference}>{proofLine.reference}</a></td>
              <td class="border border-gray-600 py-1 px-2">{proofLine.assertion}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
