<script lang="ts" module>
  import SettingsTabComponent from "$lib/components/tabs/SettingsTabComponent.svelte";

  export class SettingsTab extends Tab {
    component = SettingsTabComponent;

    async loadData(): Promise<void> {}

    unloadData(): void {}

    name(): string {
      return "Settings";
    }

    sameTab(tab: Tab) {
      return tab instanceof SettingsTab;
    }

    validTab(): boolean {
      return true;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import GeneralSettingsTabComponent from "./settingsTabs/GeneralSettingsTabComponent.svelte";
  import type { Component } from "svelte";

  let { tab }: { tab: Tab } = $props();

  let settingsTab = $derived.by(() => {
    if (tab instanceof SettingsTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let tabs: { name: string; component: Component }[] = [{ name: "General", component: GeneralSettingsTabComponent }];

  let currentTabIndex = $state(0);
  let currentTab = $derived(tabs[currentTabIndex]);

  let changeTab = (index: number) => {
    currentTabIndex = index;
  };
</script>

<div class="w-full h-full">
  <div class="w-44 h-full fixed border-r overflow-hidden">
    <ul class="pl-2 pt-2">
      {#each tabs as tab, index}
        <li class:custom-bg-active-color={index == currentTabIndex}>
          <button class="pl-1 text-nowrap w-full text-left" onclick={() => changeTab(index)}>{tab.name}</button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="ml-44 h-full overflow-y-auto">
    <currentTab.component />
  </div>
</div>
