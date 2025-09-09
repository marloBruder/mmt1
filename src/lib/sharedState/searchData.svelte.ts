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

let searchParameters: SearchParameters = $state(getNewDefaultSearchParamters());

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

let nextSearchNumber = $state(1);

function getNextSearchNumber(): number {
  nextSearchNumber += 1;
  return nextSearchNumber - 1;
}

export { searchParameters, searchInputValues, getNextSearchNumber, defaultSearchParameters };
