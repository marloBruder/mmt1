<script lang="ts">
  import type { MouseEventHandler } from "svelte/elements";

  let {
    onclick = () => {},
    ondblclick = () => {},
    children,
    disabled,
    confirm,
    warning,
  }: {
    onclick?: MouseEventHandler<HTMLButtonElement>;
    ondblclick?: MouseEventHandler<HTMLButtonElement>;
    children: any;
    disabled?: boolean;
    confirm?: boolean;
    warning?: boolean;
  } = $props();
</script>

<button class="border rounded px-1 disabled:text-gray-700" class:bg-red-400={warning} class:bg-green-700={confirm} {onclick} {ondblclick} {disabled}>{@render children()}</button>
