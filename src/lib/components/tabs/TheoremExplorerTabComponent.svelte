<script lang="ts" module>
  import TheoremExplorerTabComponent from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import type { TheoremListData } from "$lib/sharedState/model.svelte";

  export class TheoremExplorerTab extends Tab {
    component = TheoremExplorerTabComponent;

    #page: number = $state(0);
    #theoremListData: TheoremListData = $state({ list: [], pageAmount: 0, theoremAmount: 0, pageLimits: null });
    #scrollToId: string | undefined;
    firstOpen: boolean = true;

    constructor(page: number, scrollToId?: string) {
      super();
      this.#page = page;
      this.#scrollToId = scrollToId;
    }

    async loadData(): Promise<void> {
      this.#theoremListData = await invoke("get_theorem_list", { page: this.#page });
    }

    unloadData(): void {
      this.#theoremListData = { list: [], pageAmount: 0, theoremAmount: 0, pageLimits: null };
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

    get scrollToId() {
      return this.#scrollToId;
    }
  }
</script>

<script lang="ts">
  import { tabManager } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";
  import { Tab } from "$lib/sharedState/tab.svelte";
  import { emit } from "@tauri-apps/api/event";

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

  $effect(() => {
    if (theoremExplorerTab) {
      requestAnimationFrame(() => {
        if (theoremExplorerTab.firstOpen && theoremExplorerTab.scrollToId !== undefined) {
          let element = document.getElementById(theoremExplorerTab.scrollToId);

          if (element !== null) {
            let parentElement = element.parentElement;

            if (parentElement !== null) {
              emit("scroll-main-tab", element.getBoundingClientRect().top - parentElement.getBoundingClientRect().top);
            }
          }
        }
        theoremExplorerTab.firstOpen = false;
      });
    }
  });
</script>

<TheoremList theoremListData={theoremExplorerTab.theoremListData} {previousPageClick} {nextPageClick} {pageButtonClick} pageNum={theoremExplorerTab.page}></TheoremList>
