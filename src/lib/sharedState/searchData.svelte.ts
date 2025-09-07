import type { SearchParameters } from "./model.svelte";

let searchParameters: SearchParameters = $state({ label: "", page: 0, axiomDependencies: [], avoidAxiomDependencies: [] });

interface SearchInputValues {
  axiomDependenciesInputValue: string;
  avoidAxiomDependenciesInputValue: string;
}

let searchInputValues: SearchInputValues = $state({ avoidAxiomDependenciesInputValue: "", axiomDependenciesInputValue: "" });

export { searchParameters, searchInputValues };
