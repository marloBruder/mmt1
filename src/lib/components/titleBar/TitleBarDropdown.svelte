<script lang="ts">
  import { type Snippet } from "svelte";

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
  <button id={buttonID} {onclick} {onfocusout} class="px-1 rounded">
    {title}
  </button>
  {#if open}
    <div id={dropdownID} class="fixed custom-bg-dropdown-color border border-black rounded-lg p-2 z-50">
      {@render children()}
    </div>
  {/if}
</div>
