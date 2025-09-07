<script lang="ts">
  import { SearchTab } from "$lib/components/tabs/SearchTabComponent.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AutocompleteListInput from "./search/AutocompleteListInput.svelte";
  import { searchInputValues, searchParameters } from "$lib/sharedState/searchData.svelte";

  let searchClick = async () => {
    tabManager.openTab(new SearchTab({ ...searchParameters }), true);
  };

  let axiomDependenciesAutocomplete = async (query: string, items: string[]) => {
    return (await invoke("axiom_autocomplete", { query, items })) as [boolean, string[]];
  };
</script>

<div class="p-2">
  <div>
    <label for="search-input">Label:</label>
    <br />
    <input id="search-input" class="border border-gray-300 rounded custom-bg-input-color" bind:value={searchParameters.label} autocomplete="off" spellcheck="false" />
  </div>
  <div class="pt-2">
    <RoundButton onclick={searchClick}>Search</RoundButton>
  </div>
  <div class="pt-2">
    Must depend on axioms:
    <AutocompleteListInput bind:items={searchParameters.axiomDependencies} bind:inputValue={searchInputValues.axiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
    Must not depend on axioms:
    <AutocompleteListInput bind:items={searchParameters.avoidAxiomDependencies} bind:inputValue={searchInputValues.avoidAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
  </div>
</div>
