<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import { type Snippet } from "svelte";

  let {
    title,
    buttonContent,
    dropdownContent,
    alignDropdownLeft = false,
    disabled = false,
    open = $bindable(false),
    onmouseenter = () => {},
    customOnclose,
    border = false,
  }: {
    title: string;
    buttonContent?: Snippet;
    dropdownContent: Snippet;
    alignDropdownLeft?: boolean;
    disabled?: boolean;
    open?: boolean;
    onmouseenter?: () => void;
    customOnclose?: () => void;
    border?: boolean;
  } = $props();

  let buttonID = "dropdownButton-id-" + createInstanceId();
  let dropdownID = "dropdown-id-" + createInstanceId();

  let onclick = () => {
    open = !open;
  };

  let onfocusout = () => {
    setTimeout(() => {
      if (customOnclose === undefined) {
        open = false;
      } else {
        customOnclose();
      }
    }, 200);
  };

  $effect(() => {
    if (open && alignDropdownLeft) {
      let rightSide = document.body.getBoundingClientRect().right - document.getElementById(buttonID)!.getBoundingClientRect().right;
      document.getElementById(dropdownID)!.style.right = rightSide + "px";
    }
  });
</script>

<div class="inline-block">
  <button id={buttonID} class={"relative rounded disabled:text-gray-500 " + (border ? " border " : "")} {onclick} {onfocusout} {onmouseenter} {title} {disabled}>
    <div class={"custom-bg-hover-color rounded-md " + (open ? " custom-bg-hover-color-without-hover-condition " : "")}>
      {#if buttonContent !== undefined}
        {@render buttonContent()}
      {:else}
        <div class="px-1">
          {title}
        </div>
      {/if}
    </div>
  </button>
  {#if open}
    <div id={dropdownID} class="fixed left-auto custom-bg-dropdown-color border border-black rounded-lg py-2 z-50">
      {@render dropdownContent()}
    </div>
  {/if}
</div>
