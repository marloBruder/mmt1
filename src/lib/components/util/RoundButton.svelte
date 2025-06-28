<script lang="ts">
  import type { MouseEventHandler } from "svelte/elements";

  let {
    onclick = () => {},
    ondblclick = () => {},
    children,
    disabled,
    warning,
  }: {
    onclick?: MouseEventHandler<HTMLButtonElement>;
    ondblclick?: MouseEventHandler<HTMLButtonElement>;
    children: any;
    disabled?: boolean;
    warning?: boolean;
  } = $props();
</script>

<button class="border border-gray-300 rounded px-1 disabled:bg-gray-300" class:bg-red-400={warning} {onclick} {ondblclick} {disabled}>{@render children()}</button>
