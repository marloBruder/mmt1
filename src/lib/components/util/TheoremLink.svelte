<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { TheoremTab } from "../tabs/TheoremTabComponent.svelte";
  import TheoremNumber from "./TheoremNumber.svelte";

  let { label, text, disabled = false, theoremNumber, noUnderline = false }: { label: string; text?: string; disabled?: boolean; theoremNumber?: number; noUnderline?: boolean } = $props();

  let onclick = (e: MouseEvent) => {
    tabManager.changeTab(new TheoremTab(label));
  };

  let onmouseup = (e: MouseEvent) => {
    if (e.button == 1) {
      e.preventDefault();
      tabManager.makeOpenTempTabPermanent();
      tabManager.openTab(new TheoremTab(label), true);
    }
  };

  let onmousedown = (e: MouseEvent) => {
    if (e.button == 1) {
      e.preventDefault();
    }
  };
</script>

<button class={(disabled ? "text-gray-400 " : "") + (noUnderline ? "" : " underline ")} {onclick} {onmousedown} {onmouseup} {disabled}>
  {#if text}
    {text}
  {:else}
    {label}
  {/if}
</button>
{#if theoremNumber !== undefined}
  <TheoremNumber {theoremNumber}></TheoremNumber>
{/if}
