<script lang="ts" module>
  import type { VariablesPageData } from "$lib/sharedState/model.svelte";
  import VariablesTabComponent from "$lib/components/tabs/VariablesTabComponent.svelte";

  export class VariablesTab extends Tab {
    component = VariablesTabComponent;

    #anyVariable: string;
    #pageData: VariablesPageData = $state({ variables: [], discriminator: "VariablesPageData" });
    #moreThanOneVariable: boolean = $state(false);

    constructor(anyVariable: string) {
      super();
      this.#anyVariable = anyVariable;
    }

    async loadData(): Promise<void> {
      this.#pageData = (await invoke("get_variable_statement", { anyVariable: this.#anyVariable })) as VariablesPageData;
      if (this.#pageData.variables.length > 1) {
        this.#moreThanOneVariable = true;
      }
    }

    unloadData(): void {
      this.#pageData = { variables: [], discriminator: "VariablesPageData" };
    }

    name(): string {
      if (this.#moreThanOneVariable) {
        return "Variables: " + this.#anyVariable + ", ...";
      } else {
        return "Variable: " + this.#anyVariable;
      }
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof VariablesTab && tab.anyVariable == this.#anyVariable;
    }

    get anyVariable() {
      return this.#anyVariable;
    }
    get pageData() {
      return this.#pageData;
    }
    get moreThanOneVariable() {
      return this.#moreThanOneVariable;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tab.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import VariablesPage from "../pages/VariablesPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let variablesTab: VariablesTab = $derived.by(() => {
    if (tab instanceof VariablesTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<VariablesPage pageData={variablesTab.pageData}></VariablesPage>
