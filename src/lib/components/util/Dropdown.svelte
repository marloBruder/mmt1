<script lang="ts">
  import { type Snippet } from "svelte";

  let { title, buttonContent, dropdownContent, alignDropdownLeft = false }: { title: string; buttonContent: Snippet; dropdownContent: Snippet; alignDropdownLeft?: boolean } = $props();

  let buttonID = $derived("dropdownButton-" + title);
  let dropdownID = $derived("dropdown-" + title);

  let open: boolean = $state(false);

  let onclick = () => {
    open = !open;
  };

  let onfocusout = () => {
    setTimeout(() => {
      open = false;
    }, 200);
  };

  $effect(() => {
    if (open && alignDropdownLeft) {
      let rightSide = document.body.getBoundingClientRect().right - document.getElementById(buttonID)!.getBoundingClientRect().right;
      document.getElementById(dropdownID)!.style.right = rightSide + "px";
    }
  });
</script>

<div>
  <button id={buttonID} {onclick} {onfocusout} {title} class="relative rounded">
    {@render buttonContent()}
  </button>
  {#if open}
    <div id={dropdownID} class="fixed left-auto custom-bg-dropdown-color border border-black rounded-lg py-2 z-50">
      {@render dropdownContent()}
    </div>
  {/if}
</div>
