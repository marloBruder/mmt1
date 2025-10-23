<script lang="ts">
  import type { HeaderPageData } from "$lib/sharedState/model.svelte";
  import DescriptionParsed from "../util/DescriptionParsed.svelte";

  let { pageData, externalWindow = false }: { pageData: HeaderPageData; externalWindow?: boolean } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">{pageData.headerPath + ": " + pageData.title}</h1>
  </div>
  <div>
    <DescriptionParsed descriptionParsed={pageData.descriptionParsed} {externalWindow}></DescriptionParsed>
  </div>
</div>
