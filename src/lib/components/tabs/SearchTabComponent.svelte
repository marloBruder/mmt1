<script lang="ts" module>
  import SearchTabComponent from "$lib/components/tabs/SearchTabComponent.svelte";
  import type { SearchParameters, TheoremListData } from "$lib/sharedState/model.svelte";

  export class SearchTab extends Tab {
    component = SearchTabComponent;

    #searchParameters: SearchParameters = $state({ page: 0, label: "", axiomDependencies: [], avoidAxiomDependencies: [] });
    #searchResult: TheoremListData = $state({ list: [], pageAmount: 0, pageLimits: null });

    constructor(searchParameters: SearchParameters) {
      super();
      this.#searchParameters = searchParameters;
    }

    async loadData(): Promise<void> {
      this.#searchResult = (await invoke("search_theorems", { searchParameters: this.#searchParameters })) as TheoremListData;
    }

    unloadData(): void {
      this.#searchResult = { list: [], pageAmount: 0, pageLimits: null };
    }

    name(): string {
      return "Search: " + this.#searchParameters.label;
    }

    sameTab(_tab: Tab): boolean {
      return false;
    }

    get searchParameters() {
      return this.#searchParameters;
    }

    get searchResult() {
      return this.#searchResult;
    }
  }
</script>

<script lang="ts">
  import { Tab, tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let searchTab = $derived.by(() => {
    if (tab instanceof SearchTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let previousPageClick = () => {
    let searchParams = { ...searchTab.searchParameters, page: searchTab.searchParameters.page - 1 };
    tabManager.changeTab(new SearchTab(searchParams));
  };

  let nextPageClick = () => {
    let searchParams = { ...searchTab.searchParameters, page: searchTab.searchParameters.page + 1 };
    tabManager.changeTab(new SearchTab(searchParams));
  };

  let pageButtonClick = (pageNum: number) => {
    let searchParams = { ...searchTab.searchParameters, page: pageNum };
    tabManager.changeTab(new SearchTab(searchParams));
  };
</script>

<TheoremList theoremListData={searchTab.searchResult} {previousPageClick} {nextPageClick} {pageButtonClick} pageNum={searchTab.searchParameters.page}></TheoremList>
