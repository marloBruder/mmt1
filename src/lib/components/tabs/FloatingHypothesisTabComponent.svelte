<script lang="ts" module>
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import FloatingHypothesisTabComponent from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import type { FloatingHypothesisPageData } from "$lib/sharedState/model.svelte";
  import { invoke } from "@tauri-apps/api/core";

  export class FloatingHypothesisTab extends Tab {
    component = FloatingHypothesisTabComponent;

    #label: string;
    #pageData: FloatingHypothesisPageData = $state({ floatingHypothesis: { label: "", typecode: "", variable: "" }, discriminator: "FloatingHypothesisPageData" });

    constructor(label: string) {
      super();
      this.#label = label;
    }

    async loadData(): Promise<void> {
      this.#pageData = (await invoke("get_floating_hypothesis_page_data_local", { label: this.#label })) as FloatingHypothesisPageData;
    }

    unloadData(): void {
      this.#pageData = { floatingHypothesis: { label: "", typecode: "", variable: "" }, discriminator: "FloatingHypothesisPageData" };
    }

    name(): string {
      return this.#label;
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof FloatingHypothesisTab && tab.label == this.#label;
    }

    get label() {
      return this.#label;
    }

    get pageData() {
      return this.#pageData;
    }
  }
</script>

<script lang="ts">
  import FloatingHypothesisPage from "../pages/FloatingHypothesisPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let floatingHypothesisTab = $derived.by(() => {
    if (tab instanceof FloatingHypothesisTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<FloatingHypothesisPage pageData={floatingHypothesisTab.pageData}></FloatingHypothesisPage>
