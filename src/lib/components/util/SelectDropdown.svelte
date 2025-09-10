<script module>
  export interface SelectDropdownOption {
    label: string;
    value: string;
  }
</script>

<script lang="ts">
  import Dropdown from "./Dropdown.svelte";

  let { options, value = $bindable() }: { options: SelectDropdownOption[]; value: string } = $props();

  let selectedLabel = $derived.by(() => {
    for (let option of options) {
      if (option.value == value) {
        return option.label;
      }
    }
    return "";
  });

  let onDropdownClick = (i: number) => {
    value = options[i].value;
  };
</script>

<Dropdown title={selectedLabel} border>
  {#snippet buttonContent()}
    <div class="px-1">{selectedLabel}</div>{/snippet}
  {#snippet dropdownContent()}
    <div>
      {#each options as option, i}
        <div class="px-2 hover:bg-purple-500">
          <button onclick={() => onDropdownClick(i)}>{option.label}</button>
        </div>
      {/each}
    </div>
  {/snippet}
</Dropdown>
