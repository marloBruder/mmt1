<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { emit } from "@tauri-apps/api/event";
  import { TheoremTab } from "../tabs/TheoremTabComponent.svelte";
  import TheoremNumber from "./TheoremNumber.svelte";

  let {
    label,
    text,
    disabled = false,
    theoremNumber,
    noUnderline = false,
    openInNewTab = false,
    externalWindow = false,
  }: {
    label: string;
    text?: string;
    disabled?: boolean;
    theoremNumber?: number;
    noUnderline?: boolean;
    openInNewTab?: boolean;
    externalWindow?: boolean;
  } = $props();

  let onclick = async (e: MouseEvent) => {
    if (externalWindow) {
      emit("external-theorem-tab-opened", label);
    } else if (openInNewTab) {
      tabManager.makeOpenTempTabPermanent();
      await tabManager.openTab(new TheoremTab(label), true);
    } else {
      tabManager.changeTab(new TheoremTab(label));
    }
  };

  let onmouseup = async (e: MouseEvent) => {
    if (e.button === 1) {
      e.preventDefault();
      if (externalWindow) {
        emit("external-theorem-tab-opened", label);
      } else {
        tabManager.makeOpenTempTabPermanent();
        await tabManager.openTab(new TheoremTab(label), true);
      }
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
