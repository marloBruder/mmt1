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
  <div class="pb-2">
    <input type="checkbox" bind:checked={settingsTab.settings.colorUnicodePreview} />
    Color Unicode preview. Meaning of colors:
    <div class="flex">
      <div class="flex h-6 w-6 items-center justify-center">
        <div class="h-4 w-4 custom-confirmation-color"></div>
      </div>
      <div>The line is correct.</div>
    </div>
    <div class="flex">
      <div class="flex h-6 w-6 items-center justify-center">
        <div class="h-4 w-4 custom-confirmation-recursive-color"></div>
      </div>
      <div>The line and all lines it depends upon are correct.</div>
    </div>
    <div class="flex">
      <div class="flex h-6 w-6 items-center justify-center">
        <div class="h-4 w-4 bg-red-950"></div>
      </div>
      <div>There is an error in this cell</div>
    </div>
    <div class="flex">
      <div class="flex h-6 w-6 items-center justify-center">
        <div class="h-4 w-4 bg-red-900"></div>
      </div>
      <div>The line will be removed after unifying.</div>
    </div>
    <div class="flex">
      <div class="flex h-6 w-6 items-center justify-center">
        <div class="h-4 w-4 bg-blue-950"></div>
      </div>
      <div>The unifier will make a change in this cell.</div>
    </div>
  </div>
  <div>
    <input type="checkbox" bind:checked={settingsTab.settings.showUnifyResultInUnicodePreview} />
    Show unify result in Unicode preview
  </div>
</div>
