<script lang="ts">
  import type { DatabaseElementPageData } from "$lib/sharedState/model.svelte";
  import CommentPage from "../pages/CommentPage.svelte";
  import ConstantsPage from "../pages/ConstantsPage.svelte";
  import FloatingHypothesisPage from "../pages/FloatingHypothesisPage.svelte";
  import HeaderPage from "../pages/HeaderPage.svelte";
  import TheoremPage from "../pages/TheoremPage.svelte";
  import VariablesPage from "../pages/VariablesPage.svelte";

  let { pageData, externalWindow = false }: { pageData: DatabaseElementPageData | null; externalWindow?: boolean } = $props();
</script>

<div class="w-full h-full">
  {#if pageData != null}
    {#if pageData.discriminator == "EmptyPageData"}
      <div class="p-2">Nothing to see yet.</div>
    {:else if pageData.discriminator == "HeaderPageData"}
      <HeaderPage {pageData} {externalWindow}></HeaderPage>
    {:else if pageData.discriminator == "CommentPageData"}
      <CommentPage {pageData}></CommentPage>
    {:else if pageData.discriminator == "ConstantsPageData"}
      <ConstantsPage {pageData}></ConstantsPage>
    {:else if pageData.discriminator == "VariablesPageData"}
      <VariablesPage {pageData}></VariablesPage>
    {:else if pageData.discriminator == "FloatingHypothesisPageData"}
      <FloatingHypothesisPage {pageData}></FloatingHypothesisPage>
    {:else if pageData.discriminator == "TheoremPageData"}
      <TheoremPage {pageData} editorPreview {externalWindow}></TheoremPage>
    {/if}
  {:else}
    <div class="p-2">Resolve all syntax errors to show the unicode preview.</div>
  {/if}
</div>
