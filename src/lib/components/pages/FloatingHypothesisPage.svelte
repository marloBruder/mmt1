<script lang="ts">
  import type { FloatingHypothesisPageData } from "$lib/sharedState/model.svelte";
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { pageData }: { pageData: FloatingHypothesisPageData } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">Floating Hypothesis: {pageData.floatingHypothesis.label}</h1>
  </div>
  <div>
    <h2 class="text-xl">Statement:</h2>
    <MetamathExpression expression={pageData.floatingHypothesis.typecode + " " + pageData.floatingHypothesis.variable}></MetamathExpression>
  </div>
</div>
