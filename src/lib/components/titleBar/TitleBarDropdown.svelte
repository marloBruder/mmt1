<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import type { MouseEventHandler } from "svelte/elements";

  let { title, children }: { title: string; children: Snippet } = $props();

  let buttonID = $derived("titleBarDropdownButton-" + title);
  let dropdownID = $derived("titleBarDropdown-" + title);

  let open: boolean = $state(false);

  let onclick = () => {
    open = !open;
  };

  let onfocusout = () => {
    setTimeout(() => {
      open = false;
    }, 200);
  };
</script>

<div>
  <button id={buttonID} {onclick} {onfocusout} class={"px-1 rounded " + (open ? "bg-gray-200 " : "")}>
    {title}
  </button>
  {#if open}
    <div id={dropdownID} class="fixed bg-white border border-black p-2 z-50">
      {@render children()}
    </div>
  {/if}
</div>
