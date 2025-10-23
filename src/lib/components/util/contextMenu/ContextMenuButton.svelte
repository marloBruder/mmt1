<script lang="ts">
  import type { Snippet } from "svelte";

  let { children, onclick, disabled = false }: { children: Snippet; onclick: () => void; disabled?: boolean } = $props();
</script>

<div><button class="w-full px-2 text-left hover:enabled:bg-purple-500 disabled:text-gray-400" {onclick} {disabled}>{@render children()}</button></div>
