import type { SearchParameters } from "./model.svelte";

let searchParameters: SearchParameters = $state({ label: "", page: 0, allAxiomDependencies: [], anyAxiomDependencies: [], avoidAxiomDependencies: [] });

interface SearchInputValues {
  allAxiomDependenciesInputValue: string;
  anyAxiomDependenciesInputValue: string;
  avoidAxiomDependenciesInputValue: string;
}

let searchInputValues: SearchInputValues = $state({ allAxiomDependenciesInputValue: "", anyAxiomDependenciesInputValue: "", avoidAxiomDependenciesInputValue: "" });

export { searchParameters, searchInputValues };
