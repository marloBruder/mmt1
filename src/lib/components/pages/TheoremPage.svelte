<script lang="ts">
  import type { TheoremPageData } from "$lib/sharedState/model.svelte";
  import DescriptionParsed from "../util/DescriptionParsed.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";
  import TheoremLink from "../util/TheoremLink.svelte";
  import TheoremNumber from "../util/TheoremNumber.svelte";
  import { settingsData } from "$lib/sharedState/settingsData.svelte";
  import RoundButton from "../util/RoundButton.svelte";
  import type { TheoremTab } from "../tabs/TheoremTabComponent.svelte";

  let {
    pageData,
    editorPreview = false,
    externalWindow = false,
    theoremTab,
  }: {
    pageData: TheoremPageData;
    editorPreview?: boolean;
    externalWindow?: boolean;
    theoremTab?: TheoremTab;
  } = $props();

  let theorem = $derived(pageData.theorem);

  let proofLineBackground = (row: number, cell: number): string => {
    if (!editorPreview || !settingsData.settings.colorUnicodePreview) {
      return "";
    }

    if (pageData.previewErrors && pageData.previewErrors[row][cell]) {
      return "bg-red-950";
    }

    if (pageData.previewDeletedMarkers && pageData.previewDeletedMarkers[row]) {
      return "bg-red-900";
    }

    if (pageData.previewConfirmationsRecursive && pageData.previewConfirmationsRecursive[row]) {
      return "custom-confirmation-recursive-color";
    }

    if (pageData.previewConfirmations && pageData.previewConfirmations[row]) {
      return "custom-confirmation-color";
    }

    if (pageData.previewUnifyMarkers && pageData.previewUnifyMarkers[row][cell]) {
      return "bg-blue-950";
    }

    return "";
  };

  let toggleShowAll = async () => {
    if (theoremTab !== undefined) {
      theoremTab.showAll = !theoremTab.showAll;
      await theoremTab.loadData();
    }
  };
</script>

<div class="text-center pb-4 flex">
  {#if !editorPreview}
    <div class="w-1/5 pt-2">
      <TheoremLink text={"< Previous"} label={pageData.lastTheoremLabel ? pageData.lastTheoremLabel : ""} disabled={pageData.lastTheoremLabel === null} noUnderline></TheoremLink>
    </div>
  {/if}
  <div class={"pt-4 " + (editorPreview ? " w-full " : " w-3/5 ")}>
    <h1 class="text-3xl">
      {#if theorem.proof}Theorem
      {:else}Axiom
      {/if}
      {theorem.label}
      {#if !editorPreview}
        <TheoremNumber theoremNumber={pageData.theoremNumber} normalTextSize></TheoremNumber>
      {/if}
    </h1>
  </div>
  {#if !editorPreview}
    <div class="w-1/5 pt-2">
      <TheoremLink text={"Next >"} label={pageData.nextTheoremLabel ? pageData.nextTheoremLabel : ""} disabled={pageData.nextTheoremLabel === null} noUnderline></TheoremLink>
    </div>
  {/if}
</div>
<div class="text-center">
  {#if theorem.hypotheses.length != 0}
    <div class="pb-4 px-4">
      <h2 class="font-bold">Hyptheses:</h2>
      <div class="flex justify-center">
        <table class="text-left">
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
  <div class="pb-4 px-4">
    <h2 class="font-bold">Assertion:</h2>
    <p><MetamathExpression expression={theorem.assertion}></MetamathExpression></p>
  </div>
  {#if theorem.distincts.length != 0}
    <div class="pb-4">
      <h2 class="font-bold">Distinct variables:</h2>
      <div class="flex flex-wrap justify-center">
        {#each theorem.distincts as distinct}
          <span class="mx-2 text-nowrap">
            {#each distinct.split(" ") as distinctVar, i}
              {#if i !== 0}{","}{/if}
              <MetamathExpression expression={distinctVar}></MetamathExpression>
            {/each}
          </span>
        {/each}
      </div>
    </div>
  {/if}
  <div class="pb-4 px-8">
    <h2 class="font-bold">Description:</h2>
    <div class="flex flex-col items-center w-full">
      <div class="text-left">
        <DescriptionParsed descriptionParsed={pageData.descriptionParsed} openLinksInNewTab={editorPreview} {externalWindow} invalidHtml={pageData.invalidHtml}></DescriptionParsed>
      </div>
    </div>
  </div>
  {#if theorem.proof != null}
    {#if editorPreview}
      <div class="pb-4 break-words">
        <h2 class="font-bold">Raw Proof:</h2>
        <p>{theorem.proof}</p>
      </div>
    {/if}
    <div class="pb-4">
      <h2 class="font-bold">Proof</h2>
      {#if !pageData.proofIncomplete}
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
              <tr>
                <td class={"border border-gray-600 py-1 px-2 " + proofLineBackground(i, 0)}>
                  {proofLine.stepName}
                </td>
                <td class={"border border-gray-600 py-1 px-2 " + proofLineBackground(i, 1)}>
                  {#each proofLine.hypotheses as hypothesis, index}
                    {hypothesis + (index != proofLine.hypotheses.length - 1 ? ", " : "")}
                  {/each}
                </td>
                <td class={"border border-gray-600 py-1 px-2 " + proofLineBackground(i, 2)}>
                  {#if proofLine.referenceNumber !== null}
                    <TheoremLink label={proofLine.reference} theoremNumber={proofLine.referenceNumber} openInNewTab={editorPreview} {externalWindow}></TheoremLink>
                  {:else}
                    {proofLine.reference}
                  {/if}
                </td>
                {#snippet indentionPoints(indention: number)}
                  <span class="text-xs text-gray-300">
                    {#each { length: indention - 1 } as _}
                      {". "}
                    {/each}
                    {indention}
                  </span>
                {/snippet}
                {#if proofLine.oldAssertion !== null && proofLine.oldAssertion !== proofLine.assertion}
                  <td class="border border-gray-600 p-0">
                    <div class="py-1 pr-2 pl-1 border-b">
                      {@render indentionPoints(proofLine.indention)}
                      <MetamathExpression expression={proofLine.oldAssertion}></MetamathExpression>
                    </div>
                    <div class={"py-1 pr-2 pl-1 " + proofLineBackground(i, 3)}>
                      {@render indentionPoints(proofLine.indention)}
                      <MetamathExpression expression={proofLine.assertion}></MetamathExpression>
                    </div>
                  </td>
                {:else}
                  <td class={"border border-gray-600 py-1 pr-2 pl-1 " + proofLineBackground(i, 3)}>
                    {@render indentionPoints(proofLine.indention)}
                    <MetamathExpression expression={proofLine.assertion}></MetamathExpression>
                  </td>
                {/if}
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        The proof is incomplete.
      {/if}
    </div>
    {#if !pageData.proofIncomplete && !editorPreview}
      <div class="px-4">
        <RoundButton onclick={toggleShowAll}>Toggle Show All Proof Steps</RoundButton>
      </div>
    {/if}
  {/if}
  <div class="p-8">
    <hr />
  </div>
  <div class="text-left px-4 pb-2">
    {#if theorem.proof !== null}
      <span class="font-bold">This theorem was proved from axioms: </span>
      {#if pageData.axiomDependencies.length === 0}
        (None)
      {/if}
      <div class="text-justify">
        {#each pageData.axiomDependencies as [axiomDependency, dependencyNumber]}
          <span class="mr-2">
            <TheoremLink label={axiomDependency} theoremNumber={dependencyNumber} openInNewTab={editorPreview} {externalWindow}></TheoremLink>
          </span>
        {/each}
      </div>
      <span class="font-bold">This theorem depends on definitions: </span>
      {#if pageData.definitionDependencies.length === 0}
        (None)
      {/if}
      <div class="text-justify">
        {#each pageData.definitionDependencies as [definitionDependency, dependencyNumber]}
          <span class="mr-2">
            <TheoremLink label={definitionDependency} theoremNumber={dependencyNumber} openInNewTab={editorPreview} {externalWindow}></TheoremLink>
          </span>
        {/each}
      </div>
    {/if}
    {#if !editorPreview}
      <span class="font-bold">This theorem is referenced by:</span>
      {#if pageData.references.length === 0}
        (None)
      {/if}
      <div class="text-justify">
        {#each pageData.references as [reference, referenceNumber]}
          <span class="mr-2">
            <TheoremLink label={reference} theoremNumber={referenceNumber} openInNewTab={editorPreview} {externalWindow}></TheoremLink>
          </span>
        {/each}
      </div>
    {/if}
  </div>
</div>
