<script lang="ts">
  import { SearchTab } from "$lib/components/tabs/SearchTabComponent.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AutocompleteListInput from "./search/AutocompleteListInput.svelte";
  import { getNextSearchNumber, searchInputValues, searchParameters } from "$lib/sharedState/searchData.svelte";
  import SearchAccordion from "./search/SearchAccordion.svelte";

  let searchClick = async () => {
    tabManager.openTab(new SearchTab({ ...searchParameters }, getNextSearchNumber()), true);
  };

  let axiomDependenciesAutocomplete = async (query: string, items: string[]) => {
    return (await invoke("axiom_autocomplete", { query, items })) as [boolean, string[]];
  };
</script>

<div class="py-2">
  <div class="p-2">
    <RoundButton onclick={searchClick} additionalClasses="w-full">Search</RoundButton>
  </div>
  <div class="pt-2">
    <SearchAccordion title="LABEL" active={searchParameters.label.length != 0}>
      <div class="px-2 pb-2">
        <label for="search-input">Label:</label>
        <br />
        <input id="search-input" class="border border-gray-300 rounded custom-bg-input-color w-full" bind:value={searchParameters.label} autocomplete="off" spellcheck="false" />
      </div>
    </SearchAccordion>
    <SearchAccordion title="AXIOM DEPENDENCIES" active={searchParameters.allAxiomDependencies.length + searchParameters.anyAxiomDependencies.length + searchParameters.avoidAxiomDependencies.length != 0}>
      <div class="px-2">
        <div class="pb-2">
          Must depend on all of the axioms:
          <AutocompleteListInput bind:items={searchParameters.allAxiomDependencies} bind:inputValue={searchInputValues.allAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must depend on one of the axioms:
          <AutocompleteListInput bind:items={searchParameters.anyAxiomDependencies} bind:inputValue={searchInputValues.anyAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must not depend on axioms:
          <AutocompleteListInput bind:items={searchParameters.avoidAxiomDependencies} bind:inputValue={searchInputValues.avoidAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
      </div>
    </SearchAccordion>
  </div>
</div>
