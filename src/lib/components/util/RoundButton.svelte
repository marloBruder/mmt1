<script lang="ts">
  import type { MouseEventHandler } from "svelte/elements";

  let {
    onclick = () => {},
    children,
    disabled,
    warning,
  }: {
    onclick?: MouseEventHandler<HTMLButtonElement>;
    children: any;
    disabled?: boolean;
    warning?: boolean;
  } = $props();
</script>

<button class="border border-black rounded px-1 disabled:bg-gray-300" class:bg-red-400={warning} {onclick} {disabled}>{@render children()}</button>
