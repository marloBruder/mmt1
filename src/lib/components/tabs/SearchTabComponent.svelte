<script lang="ts" module>
  import SearchTabComponent from "$lib/components/tabs/SearchTabComponent.svelte";
  import type { TheoremListData } from "$lib/sharedState/model.svelte";

  export class SearchTab extends Tab {
    component = SearchTabComponent;

    #searchParameters: SearchParameters = $state(defaultSearchParameters);
    #searchResult: TheoremListData = $state({ list: [], pageAmount: 0, pageLimits: null });
    #searchNumber: number = $state(0);

    constructor(searchParameters: SearchParameters, searchNumber: number) {
      super();
      this.#searchParameters = searchParameters;
      this.#searchNumber = searchNumber;
    }

    async loadData(): Promise<void> {
      this.#searchResult = (await invoke("search_theorems", { searchParameters: this.#searchParameters })) as TheoremListData;
    }

    unloadData(): void {
      this.#searchResult = { list: [], pageAmount: 0, pageLimits: null };
    }

    name(): string {
      return "Search #" + this.#searchNumber;
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

    get searchNumber() {
      return this.#searchNumber;
    }
  }
</script>

<script lang="ts">
  import { Tab, tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";
  import { defaultSearchParameters, type SearchParameters } from "$lib/sharedState/searchData.svelte";

  let { tab }: { tab: Tab } = $props();

  let searchTab = $derived.by(() => {
    if (tab instanceof SearchTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let previousPageClick = () => {
    let searchParams = { ...searchTab.searchParameters, page: searchTab.searchParameters.page - 1 };
    tabManager.changeTab(new SearchTab(searchParams, searchTab.searchNumber));
  };

  let nextPageClick = () => {
    let searchParams = { ...searchTab.searchParameters, page: searchTab.searchParameters.page + 1 };
    tabManager.changeTab(new SearchTab(searchParams, searchTab.searchNumber));
  };

  let pageButtonClick = (pageNum: number) => {
    let searchParams = { ...searchTab.searchParameters, page: pageNum };
    tabManager.changeTab(new SearchTab(searchParams, searchTab.searchNumber));
  };
</script>

<TheoremList theoremListData={searchTab.searchResult} {previousPageClick} {nextPageClick} {pageButtonClick} pageNum={searchTab.searchParameters.page}></TheoremList>
