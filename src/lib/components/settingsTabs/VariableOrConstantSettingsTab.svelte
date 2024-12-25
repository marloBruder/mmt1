<script lang="ts">
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import type { Constant, Variable } from "$lib/sharedState/model.svelte";
  import type { SettingsTab } from "$lib/sharedState/tabData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { constantsTab, tab }: { constantsTab: boolean; tab: SettingsTab } = $props();

  let data: { symbol: string }[] = $derived(constantsTab ? tab.constants : tab.variables);

  let editing = $state(false);

  let text = $state("");

  let editData = () => {
    let newText = "";
    let keyword = constantsTab ? "$c" : "$v";
    for (let varOrCon of data) {
      newText = newText + keyword + " " + varOrCon.symbol + " $.\n";
    }
    text = newText;
    editing = true;
  };

  let saveData = async () => {
    let command = constantsTab ? "text_to_constants" : "text_to_variables";
    invoke(command, { text }).then((newDataUnknown) => {
      if (constantsTab) {
        tab.constants = newDataUnknown as Constant[];
      } else {
        tab.variables = newDataUnknown as Variable[];
      }
      editing = false;
    });
  };
</script>

<div class="p-2">
  <div class="text-2xl pb-4">
    <h1>
      {#if constantsTab}
        Constants:
      {:else}
        Variables:
      {/if}
    </h1>
  </div>
  <div class="pb-2">
    <RoundButton onclick={editData} disabled={editing}>Edit</RoundButton>
    <RoundButton onclick={saveData} disabled={!editing}>Save</RoundButton>
  </div>
  {#if !editing}
    <div class="pl-4">
      <ol class="list-decimal">
        {#each data as conOrVar}
          <li>{conOrVar.symbol}</li>
        {/each}
      </ol>
    </div>
  {:else}
    <div>
      <textarea bind:value={text} class="w-full h-96 border border-black rounded"></textarea>
    </div>
  {/if}
</div>
