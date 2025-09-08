<script lang="ts">
  import { onMount } from "svelte";

  let { items = $bindable(), inputValue = $bindable(), autocomplete }: { items: string[]; inputValue: string; autocomplete: (query: string, items: string[]) => Promise<[boolean, string[]]> } = $props();

  let autocompleteItems: string[] = $state([]);
  let autocompleteFocus: number = $state(-1);

  let lastInputValueValid: boolean = $state(false);
  let lastInputValue: string = $state("");

  let onInputKeydown = async (e: KeyboardEvent) => {
    if (e.key === "Enter") {
      if (autocompleteFocus === -1) {
        // prevents a bug where lastInputValueValid is not updated fast enough
        let [inputValueValid, _] = await autocomplete(inputValue, items);

        if (inputValueValid) {
          items.push(inputValue);
          inputValue = "";
          lastInputValue = "";
          autocompleteItems = [];
          autocompleteFocus = -1;
        }
      } else {
        items.push(autocompleteItems[autocompleteFocus]);
        inputValue = "";
        lastInputValue = "";
        autocompleteItems = [];
        autocompleteFocus = -1;
      }
    } else if (e.key === "ArrowDown" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      if (autocompleteFocus === autocompleteItems.length - 1) {
        autocompleteFocus = -1;
        inputValue = lastInputValue;
      } else {
        autocompleteFocus += 1;
        inputValue = autocompleteItems[autocompleteFocus];
      }
    } else if (e.key === "ArrowUp" || (e.key === "Tab" && e.shiftKey)) {
      e.preventDefault();
      if (autocompleteFocus === -1) {
        autocompleteFocus = autocompleteItems.length - 1;
        inputValue = autocompleteItems[autocompleteFocus];
      } else if (autocompleteFocus === 0) {
        autocompleteFocus = -1;
        inputValue = lastInputValue;
      } else {
        autocompleteFocus -= 1;
        inputValue = autocompleteItems[autocompleteFocus];
      }
    }
  };

  let oninput = async () => {
    if (inputValue !== "") {
      [lastInputValueValid, autocompleteItems] = await autocomplete(inputValue, items);
    } else {
      autocompleteItems = [];
    }
    autocompleteFocus = -1;
    lastInputValue = inputValue;
  };

  let onRemoveClick = async (i: number) => {
    items.splice(i, 1);
    // update autocompleteItems when an item is removed
    if (inputValue !== "") {
      [lastInputValueValid, autocompleteItems] = await autocomplete(inputValue, items);
      autocompleteFocus = -1;
    }
  };

  let onAutocompleteClick = (i: number) => {
    if (i === -1) {
      if (lastInputValueValid) {
        items.push(lastInputValue);
        inputValue = "";
        lastInputValue = "";
        autocompleteItems = [];
        autocompleteFocus = -1;
      }
    } else {
      items.push(autocompleteItems[i]);
      inputValue = "";
      lastInputValue = "";
      autocompleteItems = [];
      autocompleteFocus = -1;
    }
  };

  onMount(async () => {
    if (inputValue !== "") {
      [lastInputValueValid, autocompleteItems] = await autocomplete(inputValue, items);
      lastInputValue = inputValue;
    }
  });
</script>

<div class="custom-bg-dropdown-color rounded">
  <div class="border rounded overflow-hidden custom-bg-color">
    {#if items.length !== 0}
      <div class="flex flex-row gap-2 flex-wrap p-2">
        {#each items as item, i}
          <div class="flex flex-row rounded-lg bg-purple-500">
            <div class="px-2 text-nowrap">
              {item}
            </div>
            <div class="w-6 text-center"><button onclick={() => onRemoveClick(i)}>X</button></div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="p-2">(None)</div>
    {/if}
    <input class="w-full custom-bg-input-color border rounded" bind:value={inputValue} onkeydown={onInputKeydown} {oninput} spellcheck="false" />
  </div>
  <div>
    {#if lastInputValue !== ""}
      <button class={"w-full pl-2 text-left " + (autocompleteFocus === -1 ? " bg-purple-500 " : "") + (lastInputValueValid ? " text-green-500 " : " text-red-500 ")} onclick={() => onAutocompleteClick(-1)}>
        {lastInputValue}
      </button>
    {/if}
    {#each autocompleteItems as item, i}
      <button class={"w-full pl-2 text-left text-green-500 " + (autocompleteFocus === i ? " bg-purple-500 " : " hover:bg-purple-400 ")} onclick={() => onAutocompleteClick(i)}>
        {item}
      </button>
    {/each}
  </div>
</div>
