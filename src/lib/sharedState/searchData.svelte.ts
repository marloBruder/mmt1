export interface SearchParameters {
  page: number;
  label: string;
  searchBySubstitution: SearchBySubstitutionCondition[];
  allAxiomDependencies: string[];
  anyAxiomDependencies: string[];
  avoidAxiomDependencies: string[];
  allDefinitionDependencies: string[];
  anyDefinitionDependencies: string[];
  avoidDefinitionDependencies: string[];
  allowTheorems: boolean;
  allowAxioms: boolean;
  allowDefinitions: boolean;
  allowSyntaxAxioms: boolean;
}

export interface SearchBySubstitutionCondition {
  searchTarget: "anyHypothesis" | "allHpotheses" | "assertion";
  match: "matches" | "contains";
  search: string;
}

const defaultSearchParameters: SearchParameters = {
  label: "",
  page: 0,
  searchBySubstitution: [],
  allAxiomDependencies: [],
  anyAxiomDependencies: [],
  avoidAxiomDependencies: [],
  allDefinitionDependencies: [],
  anyDefinitionDependencies: [],
  avoidDefinitionDependencies: [],
  allowTheorems: true,
  allowAxioms: true,
  allowDefinitions: true,
  allowSyntaxAxioms: true,
};

const defaultSearchBySubstitutionCondition: SearchBySubstitutionCondition = {
  searchTarget: "anyHypothesis",
  match: "matches",
  search: "",
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

export { searchData, searchInputValues, defaultSearchParameters, defaultSearchBySubstitutionCondition };
