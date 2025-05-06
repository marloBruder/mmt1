<script lang="ts" module>
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import TheoremTabComponent from "$lib/components/tabs/TheoremTabComponent.svelte";
  import type { TheoremPageData } from "$lib/sharedState/model.svelte";
  import { invoke } from "@tauri-apps/api/core";

  export class TheoremTab extends Tab {
    component = TheoremTabComponent;

    #theoremLabel: string;
    #pageData: TheoremPageData = $state({ theorem: { label: "", description: "", distincts: [], hypotheses: [], assertion: "", proof: null }, theoremNumber: 0, proofLines: [], lastTheoremLabel: null, nextTheoremLabel: null });

    constructor(theoremLabel: string) {
      super();
      this.#theoremLabel = theoremLabel;
    }

    async loadData(): Promise<void> {
      this.#pageData = await invoke("get_theorem_page_data_local", { label: this.#theoremLabel });
    }

    unloadData(): void {
      this.#pageData = { theorem: { label: "", description: "", distincts: [], hypotheses: [], assertion: "", proof: null }, theoremNumber: 0, proofLines: [], lastTheoremLabel: null, nextTheoremLabel: null };
    }

    name(): string {
      return this.#theoremLabel;
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof TheoremTab && this.#theoremLabel == tab.theoremLabel;
    }

    get pageData() {
      return this.#pageData;
    }

    get theoremLabel() {
      return this.#theoremLabel;
    }
  }
</script>

<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import TheoremLink from "../util/TheoremLink.svelte";
  import TheoremPage from "../util/TheoremPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremTab = $derived.by(() => {
    if (tab instanceof TheoremTab) {
      return tab;
    }
    throw Error("Wrong Tab Type:" + typeof tab + ", " + JSON.stringify(tab));
  });

  let pageData = $derived(theoremTab.pageData);
</script>

<TheoremPage {pageData}></TheoremPage>
