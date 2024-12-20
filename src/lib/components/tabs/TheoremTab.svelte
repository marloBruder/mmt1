<script lang="ts">
  import { theoremData } from "$lib/sharedState/metamathData/theoremData.svelte";
  import { tabManager } from "$lib/sharedState/tabData.svelte";

  let { theoremName }: { theoremName: string } = $props();

  let theorem = $derived.by(() => {
    let theoremOrNull = theoremData.getTheoremByName(theoremName);
    if (!theoremOrNull) {
      // Should never happen
      tabManager.openEmptyTab();
      return {
        name: "",
        description: "",
        disjoints: [],
        hypotheses: [],
        assertion: "",
        proof: null,
      };
    }
    return theoremOrNull;
  });
</script>

<div class="h-full w-full">
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
</div>
