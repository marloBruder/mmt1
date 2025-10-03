<script lang="ts" module>
  import SettingsTabComponent from "$lib/components/tabs/SettingsTabComponent.svelte";
  import { defaultSettings, settingsData, type Settings } from "$lib/sharedState/settingsData.svelte";

  export class SettingsTab extends Tab {
    component = SettingsTabComponent;

    settings: Settings = $state(defaultSettings);

    unsavedChanges = $derived(JSON.stringify(settingsData.settings) !== JSON.stringify(this.settings));

    databaseReloadRequired = $derived(settingsData.settings.definitionsStartWith !== this.settings.definitionsStartWith);

    async loadData(): Promise<void> {
      this.settings = settingsData.cloneSettings();
    }

    unloadData(): void {
      this.settings = defaultSettings;
    }

    name(): string {
      return "Settings";
    }

    sameTab(tab: Tab) {
      return tab instanceof SettingsTab;
    }

    validTab(): boolean {
      return true;
    }

    showUnsavedChanges(): boolean {
      return this.unsavedChanges;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import HorizontalSplit from "../util/HorizontalSplit.svelte";
  import RoundButton from "../util/RoundButton.svelte";
  import ScrollableContainer from "../util/ScrollableContainer.svelte";
  import VerticalSplit from "../util/VerticalSplit.svelte";
  import GeneralSettingsTabComponent from "./settingsTabs/GeneralSettingsTabComponent.svelte";
  import type { Component } from "svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import { goto } from "$app/navigation";
  import { confirm } from "@tauri-apps/plugin-dialog";

  let { tab }: { tab: Tab } = $props();

  let settingsTab = $derived.by(() => {
    if (tab instanceof SettingsTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let tabs: { name: string; component: Component<{ settingsTab: SettingsTab }> }[] = [
    {
      name: "General",
      component: GeneralSettingsTabComponent,
    },
  ];

  let currentTabIndex = $state(0);
  let currentTab = $derived(tabs[currentTabIndex]);

  let changeTab = (index: number) => {
    currentTabIndex = index;
  };

  let saveChanges = async () => {
    const databaseReloadRequired = settingsTab.databaseReloadRequired;

    if (databaseReloadRequired && globalState.databaseState !== null && !(await confirm("Are you sure you want to reload the database to save your settings?"))) {
      return;
    }

    settingsData.settings = util.clone(settingsTab.settings) as Settings;
    await settingsData.settingsStore?.set("settings", settingsTab.settings);
    await invoke("set_settings", { settings: settingsTab.settings });

    if (databaseReloadRequired && globalState.databaseState !== null) {
      globalState.databaseBeingOpened = globalState.databaseState.databasePath;
      goto("/main/openDatabase");
    }
  };
</script>

<div class="w-full h-full overflow-hidden">
  <HorizontalSplit secondFixed>
    {#snippet first()}
      <VerticalSplit>
        {#snippet first()}
          <div class="w-44 h-full border-r">
            <ul class="pl-2 pt-2">
              {#each tabs as tab, index}
                <li class:custom-bg-active-color={index == currentTabIndex}>
                  <button class="pl-1 text-nowrap w-full text-left" onclick={() => changeTab(index)}>{tab.name}</button>
                </li>
              {/each}
            </ul>
          </div>
        {/snippet}
        {#snippet second()}
          <div class="h-full">
            <ScrollableContainer>
              <currentTab.component {settingsTab} />
            </ScrollableContainer>
          </div>
        {/snippet}
      </VerticalSplit>
    {/snippet}
    {#snippet second()}
      <div class="w-full h-12 border-t">
        <div class="h-full flex flex-row items-center justify-between text-nowrap">
          <div class="px-6">
            {#if settingsTab.unsavedChanges}
              Unsaved Changes
            {:else}
              No Unsaved Changes
            {/if}
            {#if settingsTab.databaseReloadRequired}
              (Database reload required)
            {/if}
          </div>
          <div class="px-6">
            <RoundButton onclick={saveChanges} disabled={!settingsTab.showUnsavedChanges}>Save changes</RoundButton>
          </div>
        </div>
      </div>
    {/snippet}
  </HorizontalSplit>
</div>
