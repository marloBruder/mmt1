<script lang="ts" module>
  import SearchTabComponent from "$lib/components/tabs/SearchTabComponent.svelte";
  import type { SearchParameters, TheoremListEntry } from "$lib/sharedState/model.svelte";

  export class SearchTab extends Tab {
    component = SearchTabComponent;

    #searchParameters: SearchParameters = $state({ label: "", start: 0, amount: 100 });
    #searchResult: TheoremListEntry[] = $state([]);

    constructor(searchParameters: SearchParameters) {
      super();
      this.#searchParameters.label = searchParameters.label;
    }

    async loadData(): Promise<void> {
      this.#searchResult = await invoke("search_theorems", { searchParameters: this.#searchParameters });
    }

    async previousPage() {
      if (this.#searchParameters.start >= 100) {
        this.#searchParameters.start -= 100;
        await this.loadData();
      }
    }

    async nextPage() {
      this.#searchParameters.start += 100;
      await this.loadData();
    }

    name(): string {
      return "Search: " + this.#searchParameters.label;
    }

    sameTab(tab: Tab): boolean {
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
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let searchTab = $derived.by(() => {
    if (tab instanceof SearchTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let previousPageClick = async () => {
    await searchTab.previousPage();
  };

  let nextPageClick = async () => {
    await searchTab.nextPage();
  };
</script>

<TheoremList theoremList={searchTab.searchResult} {previousPageClick} {nextPageClick}></TheoremList>
