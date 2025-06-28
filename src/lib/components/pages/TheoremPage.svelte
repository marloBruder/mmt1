<script lang="ts">
  import type { TheoremPageData } from "$lib/sharedState/model.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";
  import TheoremLink from "../util/TheoremLink.svelte";

  let { pageData }: { pageData: TheoremPageData } = $props();

  let theorem = $derived(pageData.theorem);

  let isHypothesisName = (name: string): boolean => {
    for (let hypothesis of theorem.hypotheses) {
      if (hypothesis.label == name) {
        return true;
      }
    }
    return false;
  };

  let proofLineBackground = (i: number): string => {
    if (pageData.previewConfirmationsRecursive) {
      if (pageData.previewConfirmationsRecursive[i]) {
        return "custom-confirmation-recursive-color";
        // return "bg-green-300";
      }
    }

    if (pageData.previewConfirmations) {
      if (pageData.previewConfirmations[i]) {
        return "custom-confirmation-color";
        // return "bg-green-200";
      }
    }

    return "";
  };
</script>

<div class="text-center pb-4 flex">
  <div class="w-1/5 pt-2">
    <TheoremLink text={"< Previous"} label={pageData.lastTheoremLabel ? pageData.lastTheoremLabel : ""} disabled={pageData.lastTheoremLabel === null}></TheoremLink>
  </div>
  <div class="w-3/5 pt-4">
    <h1 class="text-3xl">
      {#if theorem.proof}Theorem
      {:else}Axiom
      {/if}
      {theorem.label}
      {#if pageData.theoremNumber != 0}
        <small class="text-sm">
          {pageData.theoremNumber}
        </small>
      {/if}
    </h1>
  </div>
  <div class="w-1/5 pt-2">
    <TheoremLink text={"Next >"} label={pageData.nextTheoremLabel ? pageData.nextTheoremLabel : ""} disabled={pageData.nextTheoremLabel === null}></TheoremLink>
  </div>
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
                <td><MetamathExpression expression={hypothesis.expression}></MetamathExpression></td>
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
  {#if theorem.distincts.length != 0}
    <div class="pb-4">
      <h2>Distinct variables:</h2>
      {#each theorem.distincts as distinct}
        <p><MetamathExpression expression={distinct}></MetamathExpression></p>
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
          {#each pageData.proofLines as proofLine, i}
            <tr class={proofLineBackground(i)}>
              <td class={"border border-gray-600 py-1 px-2 " + (pageData.previewErrors ? (pageData.previewErrors[i][0] ? " bg-red-400 " : "") : "")}>{proofLine.stepName}</td>
              <td class={"border border-gray-600 py-1 px-2 " + (pageData.previewErrors ? (pageData.previewErrors[i][1] ? " bg-red-400 " : "") : "")}>
                {#each proofLine.hypotheses as hypothesis, index}
                  {hypothesis + (index != proofLine.hypotheses.length - 1 ? ", " : "")}
                {/each}
              </td>
              <td class={"border border-gray-600 py-1 px-2 " + (pageData.previewErrors ? (pageData.previewErrors[i][2] ? " bg-red-400 " : "") : "")}>
                {#if !isHypothesisName(proofLine.reference)}
                  <TheoremLink label={proofLine.reference}></TheoremLink>
                {:else}
                  {proofLine.reference}
                {/if}
              </td>
              <td class={"border border-gray-600 py-1 pr-2" + (pageData.previewErrors ? (pageData.previewErrors[i][3] ? " bg-red-400 " : "") : "")}>
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

<style>
  .custom-confirmation-recursive-color {
    background-color: #005030;
  }
  .custom-confirmation-color {
    background-color: #003d30;
  }
</style>
