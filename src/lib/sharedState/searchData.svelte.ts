import { util } from "./util.svelte";

export interface SearchParameters {
  page: number;
  label: string;
  searchByParseTree: SearchByParseTreeCondition[];
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

export interface SearchByParseTreeCondition {
  searchTarget: "anyHypothesis" | "allHypotheses" | "assertion" | "anyExpressions" | "allExpressions";
  searchCondition: "matches" | "contains";
  search: string;
}

const defaultSearchParameters: SearchParameters = {
  label: "",
  page: 0,
  searchByParseTree: [],
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

const defaultSearchByParseTreeCondition: SearchByParseTreeCondition = {
  searchTarget: "anyHypothesis",
  searchCondition: "matches",
  search: "",
};

interface SearchInputData {
  searchByParseTreeValidInputs: boolean[];
  allAxiomDependenciesInputValue: string;
  anyAxiomDependenciesInputValue: string;
  avoidAxiomDependenciesInputValue: string;
  allDefinitionDependenciesInputValue: string;
  anyDefinitionDependenciesInputValue: string;
  avoidDefinitionDependenciesInputValue: string;
}

let defaultSearchInputData: SearchInputData = {
  searchByParseTreeValidInputs: [],
  allAxiomDependenciesInputValue: "",
  anyAxiomDependenciesInputValue: "",
  avoidAxiomDependenciesInputValue: "",
  allDefinitionDependenciesInputValue: "",
  anyDefinitionDependenciesInputValue: "",
  avoidDefinitionDependenciesInputValue: "",
};

class SearchData {
  searchParameters: SearchParameters = $state(util.clone(defaultSearchParameters));
  searchInputData: SearchInputData = $state(util.clone(defaultSearchInputData));
  nextSearchNumber = $state(1);

  resetSearchParameters() {
    this.searchParameters = util.clone(defaultSearchParameters);
    this.searchInputData = util.clone(defaultSearchInputData);
  }

  getNextSearchNumber(): number {
    this.nextSearchNumber += 1;
    return this.nextSearchNumber - 1;
  }
}

let searchData = new SearchData();

export { searchData, defaultSearchParameters, defaultSearchByParseTreeCondition };
