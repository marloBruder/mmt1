import type { SearchParameters } from "./model.svelte";

let searchParameters: SearchParameters = $state({
  label: "",
  page: 0,
  allAxiomDependencies: [],
  anyAxiomDependencies: [],
  avoidAxiomDependencies: [],
  allDefinitionDependencies: [],
  anyDefinitionDependencies: [],
  avoidDefinitionDependencies: [],
});

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

export { searchParameters, searchInputValues, getNextSearchNumber };
