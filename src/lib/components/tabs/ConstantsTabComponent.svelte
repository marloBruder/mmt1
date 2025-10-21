<script lang="ts" module>
  import type { ConstantsPageData } from "$lib/sharedState/model.svelte";
  import ConstantsTabComponent from "$lib/components/tabs/ConstantsTabComponent.svelte";

  export class ConstantsTab extends Tab {
    component = ConstantsTabComponent;

    #anyConstant: string;
    #pageData: ConstantsPageData = $state({ constants: [], discriminator: "ConstantsPageData" });
    #moreThanOneConstant: boolean = $state(false);

    constructor(anyConstant: string) {
      super();
      this.#anyConstant = anyConstant;
    }

    async loadData(): Promise<void> {
      this.#pageData = (await invoke("get_constant_statement_local", { anyConstant: this.#anyConstant })) as ConstantsPageData;
      if (this.#pageData.constants.length > 1) {
        this.#moreThanOneConstant = true;
      }
    }

    unloadData(): void {
      this.#pageData = { constants: [], discriminator: "ConstantsPageData" };
    }

    name(): string {
      if (this.#moreThanOneConstant) {
        return "Constants: " + this.#anyConstant + ", ...";
      } else {
        return "Constant: " + this.#anyConstant;
      }
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof ConstantsTab && tab.anyConstant == this.#anyConstant;
    }

    get anyConstant() {
      return this.#anyConstant;
    }
    get pageData() {
      return this.#pageData;
    }
    get moreThanOneConstant() {
      return this.#moreThanOneConstant;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tab.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ConstantsPage from "../pages/ConstantsPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let constantsTab: ConstantsTab = $derived.by(() => {
    if (tab instanceof ConstantsTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<ConstantsPage pageData={constantsTab.pageData}></ConstantsPage>
