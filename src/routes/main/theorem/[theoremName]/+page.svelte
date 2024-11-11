<script lang="ts">
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();
</script>

<div class="h-full border-8 border-blue-600">
  <p>This is Theorem {data.theoremName}</p>
</div>
