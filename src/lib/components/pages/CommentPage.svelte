<script lang="ts">
  import type { CommentPageData } from "$lib/sharedState/model.svelte";

  let { pageData }: { pageData: CommentPageData } = $props();
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">{"Comment " + pageData.commentPath}</h1>
  </div>
  <div>
    <p class="whitespace-pre-line">{pageData.comment.text}</p>
  </div>
</div>
