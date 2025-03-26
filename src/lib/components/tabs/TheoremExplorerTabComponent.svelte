<script lang="ts" module>
  import TheoremExplorerTabComponent from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
  import type { TheoremListEntry } from "$lib/sharedState/model.svelte";

  export class TheoremExplorerTab extends Tab {
    component = TheoremExplorerTabComponent;

    #start: number = $state(1);
    #theoremList: TheoremListEntry[] = $state([]);

    async loadData(): Promise<void> {
      this.#theoremList = await invoke("get_theorem_list_local", { from: this.#start, to: this.#start + 100 });
    }

    unloadData(): void {
      this.#theoremList = [];
    }

    async changePage(newStart: number) {
      this.#start = newStart;
      await this.loadData();
    }

    name(): string {
      return "Theorem Explorer";
    }

    sameTab(tab: Tab): boolean {
      return false;
    }

    get start() {
      return this.#start;
    }

    get theoremList() {
      return this.#theoremList;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import TheoremList from "../util/TheoremList.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremExplorerTab = $derived.by(() => {
    if (tab instanceof TheoremExplorerTab) {
      return tab;
    }
    throw Error("Wrong Tab Type");
  });

  let nextPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start + 100);
  };

  let previousPageClick = async () => {
    await theoremExplorerTab.changePage(theoremExplorerTab.start - 100);
  };
</script>

<TheoremList theoremList={theoremExplorerTab.theoremList} {previousPageClick} {nextPageClick}></TheoremList>
