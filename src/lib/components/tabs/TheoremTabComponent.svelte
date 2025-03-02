<script lang="ts">
  import { Tab, tabManager, TheoremTab } from "$lib/sharedState/tabManager.svelte";
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremTab = $derived.by(() => {
    if (tab instanceof TheoremTab) {
      return tab;
    }
    throw Error("Wrong Tab Type:" + typeof tab + ", " + JSON.stringify(tab));
  });

  let pageData = $derived(theoremTab.pageData);
  let theorem = $derived(pageData.theorem);

  let isHypothesisName = (name: string): boolean => {
    for (let hypothesis of theorem.hypotheses) {
      if (hypothesis.label == name) {
        return true;
      }
    }
    return false;
  };

  let refClick = (name: string) => {
    tabManager.changeTab(new TheoremTab(name));
  };
</script>

<div class="text-center py-4">
  <h1 class="text-3xl">
    {#if theorem.proof}Theorem
    {:else}Axiom
    {/if}
    {theorem.label}
    <small class="text-sm">
      {pageData.theoremNumber}
    </small>
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
                <td><MetamathExpression expression={hypothesis.hypothesis}></MetamathExpression></td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
  <div class="pb-4">
    <h2>Assertion:</h2>
    <p><MetamathExpression expression={theorem.assertion}></MetamathExpression></p>
  </div>
  {#if theorem.disjoints.length != 0}
    <div class="pb-4">
      <h2>Disjoints:</h2>
      {#each theorem.disjoints as disjoint}
        <p><MetamathExpression expression={disjoint}></MetamathExpression></p>
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
    <div class="pb-4">
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
              <td class="border border-gray-600 py-1 px-2">
                {#if !isHypothesisName(proofLine.reference)}
                  <button onclick={() => refClick(proofLine.reference)}>{proofLine.reference}</button>
                {:else}
                  {proofLine.reference}
                {/if}
              </td>
              <td class="border border-gray-600 py-1 pr-2">
                <span class="text-xs text-gray-600">
                  {#each { length: proofLine.indention - 1 } as _}
                    {". "}
                  {/each}
                  {proofLine.indention}
                </span>
                <MetamathExpression expression={proofLine.assertion}></MetamathExpression>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
