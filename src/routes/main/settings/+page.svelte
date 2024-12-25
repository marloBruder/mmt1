<script lang="ts">
  import type { PageData } from "./$types";
  import VariableOrConstantSettingsTab from "$lib/components/settingsTabs/VariableOrConstantSettingsTab.svelte";

  let { data }: { data: PageData } = $props();

  let tab = $derived(data.tab);

  let tabNames = ["Constants", "Variables"];

  let currentTab = $state(0);

  let changeTab = (index: number) => {
    currentTab = index;
  };
</script>

<div class="w-full h-full">
  <div class="w-36 h-full fixed border-r border-gray-300">
    <ul class="pl-2 pt-2">
      {#each tabNames as name, index}
        <li>
          <button onclick={() => changeTab(index)}>{name}</button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="ml-36 h-full overflow-y-auto">
    {#if currentTab === 0}
      <VariableOrConstantSettingsTab constantsTab={true} {tab}></VariableOrConstantSettingsTab>
    {:else if currentTab === 1}
      <VariableOrConstantSettingsTab constantsTab={false} {tab}></VariableOrConstantSettingsTab>
    {/if}
  </div>
</div>
