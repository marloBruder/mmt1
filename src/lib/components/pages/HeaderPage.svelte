<script lang="ts">
  import type { HeaderPageData } from "$lib/sharedState/model.svelte";

  let { pageData, externalWindow = false }: { pageData: HeaderPageData; externalWindow?: boolean } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">{pageData.headerPath + ": " + pageData.title}</h1>
  </div>
  <div>
    <p>{pageData.description}</p>
  </div>
</div>
