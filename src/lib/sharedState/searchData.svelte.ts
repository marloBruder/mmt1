import type { SearchParameters } from "./model.svelte";

const defaultSearchParameters: SearchParameters = {
  label: "",
  page: 0,
  allowTheorems: true,
  allowAxioms: true,
  allowDefinitions: true,
  allowSyntaxAxioms: true,
  allAxiomDependencies: [],
  anyAxiomDependencies: [],
  avoidAxiomDependencies: [],
  allDefinitionDependencies: [],
  anyDefinitionDependencies: [],
  avoidDefinitionDependencies: [],
};

let getNewDefaultSearchParamters = () => {
  // Clones the object
  return JSON.parse(JSON.stringify(defaultSearchParameters)) as SearchParameters;
};

class SearchData {
  searchParameters: SearchParameters = $state(getNewDefaultSearchParamters());
  nextSearchNumber = $state(1);

  resetSearchParameters() {
    this.searchParameters = getNewDefaultSearchParamters();
  }

  getNextSearchNumber(): number {
    this.nextSearchNumber += 1;
    return this.nextSearchNumber - 1;
  }
}

interface SearchInputValues {
  allAxiomDependenciesInputValue: string;
  anyAxiomDependenciesInputValue: string;
  avoidAxiomDependenciesInputValue: string;
  allDefinitionDependenciesInputValue: string;
  anyDefinitionDependenciesInputValue: string;
  avoidDefinitionDependenciesInputValue: string;
}

let searchInputValues: SearchInputValues = $state({
  allAxiomDependenciesInputValue: "",
  anyAxiomDependenciesInputValue: "",
  avoidAxiomDependenciesInputValue: "",
  allDefinitionDependenciesInputValue: "",
  anyDefinitionDependenciesInputValue: "",
  avoidDefinitionDependenciesInputValue: "",
});

let searchData = new SearchData();

export { searchData, searchInputValues, defaultSearchParameters };
