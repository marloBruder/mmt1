<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { TheoremTab } from "../tabs/TheoremTabComponent.svelte";
  import TheoremNumber from "./TheoremNumber.svelte";

  let {
    label,
    text,
    disabled = false,
    theoremNumber,
    noUnderline = false,
    openInNewTab = false,
  }: {
    label: string;
    text?: string;
    disabled?: boolean;
    theoremNumber?: number;
    noUnderline?: boolean;
    openInNewTab?: boolean;
  } = $props();

  let onclick = async (e: MouseEvent) => {
    if (openInNewTab) {
      tabManager.makeOpenTempTabPermanent();
      await tabManager.openTab(new TheoremTab(label), true);
    } else {
      tabManager.changeTab(new TheoremTab(label));
    }
  };

  let onmouseup = async (e: MouseEvent) => {
    if (e.button === 1) {
      e.preventDefault();
      tabManager.makeOpenTempTabPermanent();
      await tabManager.openTab(new TheoremTab(label), true);
    }
  };

  let onmousedown = (e: MouseEvent) => {
    if (e.button === 1) {
      e.preventDefault();
    }
  };
</script>

<button class={(disabled ? "text-gray-400 " : "") + (noUnderline ? "" : " underline ")} {onclick} {onmousedown} {onmouseup} {disabled}>
  {#if text !== undefined}
    {text}
  {:else}
    {label}
  {/if}
</button>
{#if theoremNumber !== undefined}
  <TheoremNumber {theoremNumber}></TheoremNumber>
{/if}
