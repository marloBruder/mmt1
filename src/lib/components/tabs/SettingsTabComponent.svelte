<script lang="ts">
  import type { PageData } from "../../../routes/main/settings/$types";
  import SymbolConfigSettingsTab from "$lib/components/settingsTabs/SymbolConfigSettingsTab.svelte";

  let { data }: { data: PageData } = $props();

  let tab = $derived(data.tab);

  let tabNames = ["Constants", "Variables", "Floating Hypotheses", "Html Representations"];

  let currentTab = $state(0);

  let changeTab = (index: number) => {
    currentTab = index;
  };
</script>

<div class="w-full h-full">
  <div class="w-44 h-full fixed border-r border-gray-300 overflow-hidden">
    <ul class="pl-2 pt-2">
      {#each tabNames as name, index}
        <li class:bg-gray-300={index == currentTab}>
          <button class="pl-1 text-nowrap" onclick={() => changeTab(index)}>{name}</button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="ml-44 h-full overflow-y-auto">
    {#if currentTab === 0}
      <SymbolConfigSettingsTab constantsTab {tab}></SymbolConfigSettingsTab>
    {:else if currentTab === 1}
      <SymbolConfigSettingsTab variablesTab {tab}></SymbolConfigSettingsTab>
    {:else if currentTab === 2}
      <SymbolConfigSettingsTab floatingHypothesesTab {tab}></SymbolConfigSettingsTab>
    {:else if currentTab === 3}
      <SymbolConfigSettingsTab htmlRepresentationsTab {tab}></SymbolConfigSettingsTab>
    {/if}
  </div>
</div>
