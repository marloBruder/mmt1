<script lang="ts" module>
  import TheoremExplorerTabComponent from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import type { TheoremListData, TheoremListEntry } from "$lib/sharedState/model.svelte";

  export class TheoremExplorerTab extends Tab {
    component = TheoremExplorerTabComponent;

    #page: number = $state(0);
    #theoremListData: TheoremListData = $state({ list: [], pageAmount: 0 });

    constructor(page: number) {
      super();
      this.#page = page;
    }

    async loadData(): Promise<void> {
      this.#theoremListData = await invoke("get_theorem_list_local", { page: this.#page });
    }

    unloadData(): void {
      this.#theoremListData = { list: [], pageAmount: 0 };
    }

    name(): string {
      return "Theorem Explorer Page " + (this.#page + 1);
    }

    sameTab(tab: Tab): boolean {
      return false;
    }

    get page() {
      return this.#page;
    }

    get theoremListData() {
      return this.#theoremListData;
    }
  }
</script>

<script lang="ts">
  import { Tab, tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremExplorerTab = $derived.by(() => {
    if (tab instanceof TheoremExplorerTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let previousPageClick = () => {
    tabManager.changeTab(new TheoremExplorerTab(theoremExplorerTab.page - 1));
  };

  let nextPageClick = () => {
    tabManager.changeTab(new TheoremExplorerTab(theoremExplorerTab.page + 1));
  };

  let pageButtonClick = (pageNum: number) => {
    tabManager.changeTab(new TheoremExplorerTab(pageNum));
  };
</script>

<TheoremList theoremListData={theoremExplorerTab.theoremListData} {previousPageClick} {nextPageClick} {pageButtonClick} pageNum={theoremExplorerTab.page}></TheoremList>
