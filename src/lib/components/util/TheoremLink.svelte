<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { TheoremTab } from "../tabs/TheoremTabComponent.svelte";

  let { label }: { label: string } = $props();

  let refClick = (e: MouseEvent) => {
    tabManager.changeTab(new TheoremTab(label));
  };

  let refMouseDown = (e: MouseEvent) => {
    if (e.button == 1) {
      e.preventDefault();
      tabManager.makeOpenTempTabPermanent();
      tabManager.openTab(new TheoremTab(label), true);
    }
  };
</script>

<button onclick={refClick} onmousedown={refMouseDown}>{label}</button>
