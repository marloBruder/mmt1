<script lang="ts">
  import { SearchTab } from "$lib/components/tabs/SearchTabComponent.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AutocompleteListInput from "./search/AutocompleteListInput.svelte";
  import { defaultSearchBySubstitutionCondition, searchData, searchInputValues, type SearchParameters } from "$lib/sharedState/searchData.svelte";
  import SearchAccordion from "./search/SearchAccordion.svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import SelectDropdown from "$lib/components/util/SelectDropdown.svelte";
  import { util } from "$lib/sharedState/util.svelte";
  import CloseIcon from "$lib/icons/titleBar/CloseIcon.svelte";

  let searchParameters = $derived(searchData.searchParameters);

  let searchClick = async () => {
    tabManager.openTab(new SearchTab(JSON.parse(JSON.stringify(searchParameters)) as SearchParameters, searchData.getNextSearchNumber()), true);
  };

  let resetClick = async () => {
    if (await confirm("Are you sure that you want to reset all search parameters?")) {
      searchData.resetSearchParameters();
    }
  };

  let searchBySubstituionSearchTargetOptions = [
    { label: "any hypothesis,", value: "anyHypothesis" },
    { label: "all hypotheses,", value: "allHpotheses" },
    { label: "the assertion,", value: "assertion" },
  ];

  let searchBySubstitutionMatchOptions = [
    { label: "matching", value: "matches" },
    { label: "containing", value: "contains" },
  ];

  let removeSearchBySubstitutionCondition = (i: number) => {
    searchParameters.searchBySubstitution.splice(i, 1);
  };

  let addSearchBySubstitutionCondition = () => {
    searchParameters.searchBySubstitution.push(util.clone(defaultSearchBySubstitutionCondition));
  };

  let axiomDependenciesAutocomplete = async (query: string, items: string[]) => {
    return (await invoke("axiom_autocomplete", { query, items })) as [boolean, string[]];
  };

  let definitionDependenciesAutocomplete = async (query: string, items: string[]) => {
    return (await invoke("definition_autocomplete", { query, items })) as [boolean, string[]];
  };
</script>

<div class="py-2">
  <div class="p-2">
    <RoundButton onclick={searchClick} additionalClasses="w-full">Search</RoundButton>
  </div>
  <div class="p-2">
    <RoundButton onclick={resetClick} additionalClasses="w-full">Reset Search Parameters</RoundButton>
  </div>
  <div class="pt-2">
    <SearchAccordion title="LABEL" active={searchParameters.label.length != 0}>
      <div class="px-2 pb-2">
        <label for="search-input">Label:</label>
        <br />
        <input id="search-input" class="border border-gray-300 rounded custom-bg-input-color w-full" bind:value={searchParameters.label} autocomplete="off" spellcheck="false" />
      </div>
    </SearchAccordion>
    <SearchAccordion title="SEARCH BY SUBSTITUTION" active={!searchParameters.allowTheorems || !searchParameters.allowAxioms || !searchParameters.allowDefinitions || !searchParameters.allowSyntaxAxioms}>
      <div class="p-2">
        {#each searchParameters.searchBySubstitution as searchBySubstitutionCondition, i}
          <div class="pb-2 border rounded-lg mb-2">
            <div class="border-b flex flex-row-reverse">
              <button onclick={() => removeSearchBySubstitutionCondition(i)}><CloseIcon></CloseIcon></button>
            </div>
            <div class="p-2">
              In
              <SelectDropdown bind:value={searchBySubstitutionCondition.searchTarget} options={searchBySubstituionSearchTargetOptions}></SelectDropdown>
              search for expressions
              <SelectDropdown bind:value={searchBySubstitutionCondition.match} options={searchBySubstitutionMatchOptions}></SelectDropdown>
            </div>
            <div class="px-2">
              <input class="w-full custom-bg-input-color border rounded" bind:value={searchBySubstitutionCondition.search} autocomplete="off" spellcheck="false" />
            </div>
          </div>
        {/each}
        <RoundButton additionalClasses="w-full" onclick={addSearchBySubstitutionCondition}>Add new condition</RoundButton>
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
    <SearchAccordion title="DEFINITION DEPENDENCIES" active={searchParameters.allDefinitionDependencies.length + searchParameters.anyDefinitionDependencies.length + searchParameters.avoidDefinitionDependencies.length != 0}>
      <div class="px-2">
        <div class="pb-2">
          Must depend on all of the definitions:
          <AutocompleteListInput bind:items={searchParameters.allDefinitionDependencies} bind:inputValue={searchInputValues.allDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must depend on one of the definitions:
          <AutocompleteListInput bind:items={searchParameters.anyDefinitionDependencies} bind:inputValue={searchInputValues.anyDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must not depend on definitions:
          <AutocompleteListInput bind:items={searchParameters.avoidDefinitionDependencies} bind:inputValue={searchInputValues.avoidDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
      </div>
    </SearchAccordion>
    <SearchAccordion title="STATEMENT TYPE" active={!searchParameters.allowTheorems || !searchParameters.allowAxioms || !searchParameters.allowDefinitions || !searchParameters.allowSyntaxAxioms}>
      <div class="px-2 text-nowrap">
        <div>
          <input bind:checked={searchParameters.allowTheorems} type="checkbox" />
          Allow theorems
        </div>
        <div>
          <input bind:checked={searchParameters.allowAxioms} type="checkbox" />
          Allow axioms
        </div>
        <div>
          <input bind:checked={searchParameters.allowDefinitions} type="checkbox" />
          Allow definitions
        </div>
        <div>
          <input bind:checked={searchParameters.allowSyntaxAxioms} type="checkbox" />
          Allow syntax axioms
        </div>
      </div>
    </SearchAccordion>
  </div>
</div>
