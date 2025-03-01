<script lang="ts">
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import type { Constant, FloatingHypothesis, HtmlRepresentation, Variable } from "$lib/sharedState/model.svelte";
  import type { SettingsTab } from "$lib/sharedState/tabData.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { constantsTab, variablesTab, floatingHypothesesTab, htmlRepresentationsTab, tab }: { constantsTab?: boolean; variablesTab?: boolean; floatingHypothesesTab?: boolean; htmlRepresentationsTab?: boolean; tab: SettingsTab } = $props();

  $effect(() => {
    // Add 1 if an argument is true, add 0 otherwise
    // The unary + converts booleans to numbers
    let componentTypesGiven = +(constantsTab === true) + +(variablesTab === true) + +(floatingHypothesesTab === true) + +(htmlRepresentationsTab === true);
    if (componentTypesGiven != 1) {
      throw Error("Only one of constantsTab, variablesTab and floatingHypothesesTab should be selected at a time");
    }
  });

  let editing = $state(false);

  let text = $state("");

  let editData = () => {
    let newText = "";
    if (constantsTab || variablesTab) {
      let keyword = constantsTab ? "$c" : "$v";
      let data = constantsTab ? tab.constants : tab.variables;
      for (let varOrCon of data) {
        newText = newText + keyword + " " + varOrCon.symbol + " $.\n";
      }
    } else if (floatingHypothesesTab) {
      for (let fh of tab.floatingHypotheses) {
        newText = newText + fh.label + " $f " + fh.typecode + " " + fh.variable + " $.\n";
      }
    } else if (htmlRepresentationsTab) {
      for (let hr of tab.htmlRepresentations) {
        newText = newText + 'htmldef "' + hr.symbol.replaceAll('"', '""') + '" as "' + hr.html.replaceAll('"', '""') + '";\n';
      }
    }
    text = newText;
    editing = true;
  };

  let saveData = async () => {
    let command = constantsTab ? "text_to_constants" : variablesTab ? "text_to_variables" : floatingHypothesesTab ? "text_to_floating_hypotheses" : "text_to_html_representations";
    invoke(command, { text }).then((newDataUnknown) => {
      if (constantsTab) {
        tab.constants = newDataUnknown as Constant[];
      } else if (variablesTab) {
        tab.variables = newDataUnknown as Variable[];
      } else if (floatingHypothesesTab) {
        tab.floatingHypotheses = newDataUnknown as FloatingHypothesis[];
      } else if (htmlRepresentationsTab) {
        tab.htmlRepresentations = newDataUnknown as HtmlRepresentation[];
        htmlData.loadLocal(tab.htmlRepresentations);
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
      {:else if variablesTab}
        Variables:
      {:else if floatingHypothesesTab}
        Floating Hypotheses:
      {:else if htmlRepresentationsTab}
        Html Representations:
      {/if}
    </h1>
  </div>
  <div class="pb-2">
    <RoundButton onclick={editData} disabled={editing}>Edit</RoundButton>
    <RoundButton onclick={saveData} disabled={!editing}>Save</RoundButton>
  </div>
  {#if !editing}
    <div class="pl-12">
      <ol class="list-decimal">
        {#if constantsTab}
          {#each tab.constants as constant}
            <li>{constant.symbol}</li>
          {/each}
        {:else if variablesTab}
          {#each tab.variables as variable}
            <li>{variable.symbol}</li>
          {/each}
        {:else if floatingHypothesesTab}
          {#each tab.floatingHypotheses as floatingHypothesis}
            <li>{floatingHypothesis.label + ": " + floatingHypothesis.typecode + " " + floatingHypothesis.variable}</li>
          {/each}
        {:else if htmlRepresentationsTab}
          {#each tab.htmlRepresentations as htmlRepresentation}
            <li>{htmlRepresentation.symbol + ": " + htmlRepresentation.html}</li>
          {/each}
        {/if}
      </ol>
    </div>
  {:else}
    <div>
      <textarea bind:value={text} class="w-full h-96 border border-black rounded"></textarea>
    </div>
  {/if}
</div>
