<script lang="ts">
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
  import type { SettingsTab } from "../SettingsTabComponent.svelte";

  let { settingsTab }: { settingsTab: SettingsTab } = $props();

  let parseTreeProgress = $derived.by(() => {
    if (globalState.databaseState !== null) {
      return globalState.databaseState.grammarCalculationsProgress;
    } else {
      return 0;
    }
  });
</script>

<div class="p-2">
  <div class="pb-2">
    Progress calculating parse trees:
    <ProgressBar progress={parseTreeProgress}></ProgressBar>
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
