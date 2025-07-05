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

<button class="border rounded px-1 disabled:text-gray-700" class:bg-red-400={warning} {onclick} {ondblclick} {disabled}>{@render children()}</button>
