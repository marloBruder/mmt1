<script lang="ts" module>
  import type { Constant, ConstantsPageData } from "$lib/sharedState/model.svelte";
  import ConstantsTabComponent from "$lib/components/tabs/ConstantsTabComponent.svelte";

  export class ConstantsTab extends Tab {
    component = ConstantsTabComponent;

    #anyConstant: string;
    #pageData: ConstantsPageData = $state({ constants: [] });
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
      this.#pageData = { constants: [] };
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
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { tab }: { tab: Tab } = $props();

  let constantsTab: ConstantsTab = $derived.by(() => {
    if (tab instanceof ConstantsTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">
      {#if constantsTab.moreThanOneConstant}
        Constants:
      {:else}
        Constant:
      {/if}
    </h1>
  </div>
  <div class="flex flex-col items-center">
    <table>
      <thead>
        <tr>
          <td class="p-2 border-b border-r border-black">ASCII</td>
          <td class="p-2 border-b border-l border-black">HTML</td>
        </tr>
      </thead>
      <tbody>
        {#each constantsTab.pageData.constants as constant}
          <tr>
            <td class="border-r border-black">{constant.symbol}</td>
            <td class="border-l border-black"><MetamathExpression expression={constant.symbol}></MetamathExpression></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
