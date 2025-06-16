<script lang="ts" module>
  import type { Variable, VariablesPageData } from "$lib/sharedState/model.svelte";
  import VariablesTabComponent from "$lib/components/tabs/VariablesTabComponent.svelte";

  export class VariablesTab extends Tab {
    component = VariablesTabComponent;

    #anyVariable: string;
    #pageData: VariablesPageData = $state({ variables: [] });
    #moreThanOneVariable: boolean = $state(false);

    constructor(anyVariable: string) {
      super();
      this.#anyVariable = anyVariable;
    }

    async loadData(): Promise<void> {
      this.#pageData = (await invoke("get_variable_statement_local", { anyVariable: this.#anyVariable })) as VariablesPageData;
      if (this.#pageData.variables.length > 1) {
        this.#moreThanOneVariable = true;
      }
    }

    unloadData(): void {
      this.#pageData = { variables: [] };
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
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { tab }: { tab: Tab } = $props();

  let variablesTab: VariablesTab = $derived.by(() => {
    if (tab instanceof VariablesTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">
      {#if variablesTab.moreThanOneVariable}
        Variables:
      {:else}
        Variable:
      {/if}
    </h1>
  </div>
  <div class="flex flex-col items-center">
    <table>
      <thead>
        <tr>
          <td class="p-2 border-b border-r border-black">ASCII</td>
          <td class="p-2 border-b border-r border-l border-black">HTML</td>
          <td class="p-2 border-b border-l border-black">Typecode</td>
        </tr>
      </thead>
      <tbody>
        {#each variablesTab.pageData.variables as [variable, typecode]}
          <tr>
            <td class="border-r border-black">{variable.symbol}</td>
            <td class="border-r border-l border-black"><MetamathExpression expression={variable.symbol}></MetamathExpression></td>
            <td class="border-l border-black"><MetamathExpression expression={typecode}></MetamathExpression></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
