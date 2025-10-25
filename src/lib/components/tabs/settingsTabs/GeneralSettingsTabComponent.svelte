<script lang="ts">
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import SelectDropdown, { type SelectDropdownOption } from "$lib/components/util/SelectDropdown.svelte";
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

  let grammarCalculationsError = $derived.by(() => {
    if (globalState.databaseState !== null) {
      return globalState.databaseState.grammarCalculationsError;
    } else {
      return false;
    }
  });

  let proofFormatOptions: SelectDropdownOption[] = [
    { label: "Uncompressed", value: "uncompressed" },
    { label: "Compressed", value: "compressed" },
  ];
</script>

<div class="p-2">
  <div class="pb-2">
    Progress calculating parse trees:
    <ProgressBar progress={parseTreeProgress}></ProgressBar>
  </div>
  {#if grammarCalculationsError}
    <div class="border rounded-lg p-2">
      <h2 class="text-red-600">ERROR</h2>
      An expression could not be successfully parsed. The unifier and other features will not be available.
    </div>
  {/if}
  <div class="py-2">
    <hr />
  </div>
  <div class="pb-2">
    Definition labels start with:
    <input class="border border-gray-300 rounded custom-bg-input-color w-48 max-w-full" bind:value={settingsTab.settings.definitionsStartWith} autocomplete="off" spellcheck="false" />
  </div>
  <div>
    Generated proof format:
    <SelectDropdown bind:value={settingsTab.settings.proofFormat} options={proofFormatOptions}></SelectDropdown>
  </div>
  <div>
    <input type="checkbox" bind:checked={settingsTab.settings.defaultShowAll} />
    Show all proof steps in theorem explorer by default
  </div>
</div>
