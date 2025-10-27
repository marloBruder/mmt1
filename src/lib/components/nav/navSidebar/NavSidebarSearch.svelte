<script lang="ts">
  import { SearchTab } from "$lib/components/tabs/SearchTabComponent.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AutocompleteListInput from "./search/AutocompleteListInput.svelte";
  import { defaultSearchByParseTreeCondition, searchData, type SearchParameters } from "$lib/sharedState/searchData.svelte";
  import SearchAccordion from "./search/SearchAccordion.svelte";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import { util } from "$lib/sharedState/util.svelte";
  import SearchByParseTreeConditionComponent from "./search/SearchByParseTreeConditionComponent.svelte";

  let searchParameters = $derived(searchData.searchParameters);
  let searchInputData = $derived(searchData.searchInputData);

  let searchClick = async () => {
    tabManager.openTab(new SearchTab(JSON.parse(JSON.stringify(searchParameters)) as SearchParameters, searchData.getNextSearchNumber()), true);
  };

  let resetClick = async () => {
    if (await confirm("Are you sure that you want to reset all search parameters?")) {
      searchData.resetSearchParameters();
    }
  };

  let searchByParseTreeValidInput = $derived(searchInputData.searchByParseTreeValidInputs.every((validInput) => validInput));

  let removeSearchByParseTreeCondition = (i: number) => {
    searchParameters.searchByParseTree.splice(i, 1);
    searchInputData.searchByParseTreeValidInputs.splice(i, 1);
  };

  let addSearchByParseTreeCondition = () => {
    searchParameters.searchByParseTree.push(util.clone(defaultSearchByParseTreeCondition));
    searchInputData.searchByParseTreeValidInputs.push(false);
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
    <RoundButton onclick={searchClick} additionalClasses="w-full" disabled={!searchByParseTreeValidInput}>Search</RoundButton>
  </div>
  <div class="p-2">
    <RoundButton onclick={resetClick} additionalClasses="w-full">Reset Search Parameters</RoundButton>
  </div>
  <div class="pt-2">
    <SearchAccordion title="LABEL" active={searchParameters.label.length !== 0} bind:open={searchInputData.searchAccordionOpenValues[0]}>
      <div class="px-2 pb-2">
        <label for="search-input">Label:</label>
        <br />
        <input id="search-input" class="border border-gray-300 rounded custom-bg-input-color w-full" bind:value={searchParameters.label} autocomplete="off" spellcheck="false" />
      </div>
    </SearchAccordion>
    <SearchAccordion title="SEARCH BY PARSE TREE" active={searchParameters.searchByParseTree.length !== 0} valid={searchByParseTreeValidInput} bind:open={searchInputData.searchAccordionOpenValues[1]}>
      <div class="p-2">
        {#each searchParameters.searchByParseTree as searchByParseTreeCondition, i}
          <SearchByParseTreeConditionComponent {searchByParseTreeCondition} onRemoveClick={() => removeSearchByParseTreeCondition(i)} bind:validInput={searchInputData.searchByParseTreeValidInputs[i]}></SearchByParseTreeConditionComponent>
        {/each}
        <RoundButton additionalClasses="w-full" onclick={addSearchByParseTreeCondition}>Add new condition</RoundButton>
      </div>
    </SearchAccordion>
    <SearchAccordion title="AXIOM DEPENDENCIES" active={searchParameters.allAxiomDependencies.length + searchParameters.anyAxiomDependencies.length + searchParameters.avoidAxiomDependencies.length != 0} bind:open={searchInputData.searchAccordionOpenValues[2]}>
      <div class="px-2">
        <div class="pb-2">
          Must depend on all of the axioms:
          <AutocompleteListInput bind:items={searchParameters.allAxiomDependencies} bind:inputValue={searchInputData.allAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must depend on one of the axioms:
          <AutocompleteListInput bind:items={searchParameters.anyAxiomDependencies} bind:inputValue={searchInputData.anyAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must not depend on axioms:
          <AutocompleteListInput bind:items={searchParameters.avoidAxiomDependencies} bind:inputValue={searchInputData.avoidAxiomDependenciesInputValue} autocomplete={axiomDependenciesAutocomplete}></AutocompleteListInput>
        </div>
      </div>
    </SearchAccordion>
    <SearchAccordion title="DEFINITION DEPENDENCIES" active={searchParameters.allDefinitionDependencies.length + searchParameters.anyDefinitionDependencies.length + searchParameters.avoidDefinitionDependencies.length != 0} bind:open={searchInputData.searchAccordionOpenValues[3]}>
      <div class="px-2">
        <div class="pb-2">
          Must depend on all of the definitions:
          <AutocompleteListInput bind:items={searchParameters.allDefinitionDependencies} bind:inputValue={searchInputData.allDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must depend on one of the definitions:
          <AutocompleteListInput bind:items={searchParameters.anyDefinitionDependencies} bind:inputValue={searchInputData.anyDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
        <div class="pb-2">
          Must not depend on definitions:
          <AutocompleteListInput bind:items={searchParameters.avoidDefinitionDependencies} bind:inputValue={searchInputData.avoidDefinitionDependenciesInputValue} autocomplete={definitionDependenciesAutocomplete}></AutocompleteListInput>
        </div>
      </div>
    </SearchAccordion>
    <SearchAccordion title="STATEMENT TYPE" active={!searchParameters.allowTheorems || !searchParameters.allowAxioms || !searchParameters.allowDefinitions || !searchParameters.allowSyntaxAxioms} bind:open={searchInputData.searchAccordionOpenValues[4]}>
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
