<script lang="ts">
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import type { SettingsTab } from "../SettingsTabComponent.svelte";

  let { settingsTab }: { settingsTab: SettingsTab } = $props();
</script>

<div class="p-2">
  <div class="pb-2">
    Progress calculating parse trees:
    <ProgressBar progress={globalState.databaseState.grammarCalculationsProgress}></ProgressBar>
  </div>
  <div class="py-2">
    <hr />
  </div>
  <div class="pb-2">
    Definition labels start with:
    <div>
      <input class="border border-gray-300 rounded custom-bg-input-color w-48 max-w-full" bind:value={settingsTab.settings.definitionsStartWith} autocomplete="off" spellcheck="false" />
    </div>
  </div>
</div>
