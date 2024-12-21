<script lang="ts">
  import VariableSettingsTab from "./VariableSettingsTab.svelte";

  let tabs = [
    {
      name: "Variables",
      component: VariableSettingsTab,
    },
  ];

  let currentTab = $state(0);

  let Component = $derived(tabs[currentTab].component);

  let changeTab = (index: number) => {
    currentTab = index;
  };
</script>

<div class="w-full h-full">
  <div class="w-36 h-full fixed border-r border-gray-300">
    <ul class="pl-2 pt-2">
      {#each tabs as tab, index}
        <li>
          <button onclick={() => changeTab(index)}>{tab.name}</button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="ml-36 h-full overflow-y-auto">
    <Component></Component>
  </div>
</div>
