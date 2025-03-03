<script lang="ts">
  import { onMount } from "svelte";
  import type { MouseEventHandler } from "svelte/elements";

  let { title, buttons }: { title: string; buttons: { title: string; buttonClick: MouseEventHandler<HTMLButtonElement> }[] } = $props();

  let buttonID = $derived("titleBarDropdownButton-" + title);
  let dropdownID = $derived("titleBarDropdown-" + title);

  let open: boolean = $state(false);

  let onclick = () => {
    open = !open;
  };

  let onfocusout = () => {
    setTimeout(() => {
      open = false;
    }, 150);
  };
</script>

<div>
  <button id={buttonID} {onclick} {onfocusout} class={"px-1 rounded " + (open ? "bg-gray-200 " : "")}>
    {title}
  </button>
  {#if open}
    <div id={dropdownID} class="fixed bg-white border border-black p-2 z-50">
      {#each buttons as button}
        <div>
          <button onclick={button.buttonClick}>{button.title}</button>
        </div>
      {/each}
    </div>
  {/if}
</div>
