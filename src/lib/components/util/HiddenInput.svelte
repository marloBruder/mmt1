<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";
  import type { Snippet } from "svelte";

  let { children, visible = $bindable(), validInput, previousValue, onconfirm }: { children?: Snippet; visible: boolean; validInput: (input: string) => boolean; previousValue: string; onconfirm: (newValue: string) => Promise<void> } = $props();

  let inputId = "hidden-input-id-" + createInstanceId();
  let inputValue = $state(previousValue);
  let errorWarning = $state(false);

  $effect(() => {
    if (visible) {
      let inputElement = document.getElementById(inputId);
      if (inputElement !== null) {
        inputElement.focus();
      }
    }
  });

  let inputOnFocusOut = () => {
    if (validInput(inputValue) && inputValue !== "") {
      confirm();
    } else {
      abort();
    }
  };

  let inputOnkeydown = (e: KeyboardEvent) => {
    errorWarning = false;
    if (e.key === "Escape") {
      abort();
    } else if (e.key === "Enter") {
      confirm();
    }
  };

  let confirm = async () => {
    if (visible) {
      if (validInput(inputValue) && inputValue !== "") {
        visible = false;
        let newValue = inputValue;
        inputValue = previousValue;
        errorWarning = false;
        if (previousValue !== newValue) {
          onconfirm(newValue);
        }
      } else {
        errorWarning = true;
      }
    }
  };

  let abort = () => {
    visible = false;
    inputValue = previousValue;
    errorWarning = false;
  };
</script>

{#if !visible}
  {@render children?.()}
{:else}
  <input id={inputId} bind:value={inputValue} class={"w-full custom-bg-input-color " + (errorWarning ? " border border-red-500 " : "")} onfocusout={inputOnFocusOut} onkeydown={inputOnkeydown} autocomplete="off" spellcheck="false" />
  <div class={"fixed max-w-44 text-xs text-wrap bg-red-500 " + (errorWarning ? "" : " invisible ")}>
    {#if inputValue !== ""}
      A file or folder with this name already exists at this location. Please choose a different name.
    {:else}
      Files and folders can't have empty names.
    {/if}
  </div>
{/if}
