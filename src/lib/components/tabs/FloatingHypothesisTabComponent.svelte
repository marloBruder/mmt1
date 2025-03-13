<script lang="ts" module>
  import { Tab } from "$lib/sharedState/tabManager.svelte";
  import FloatingHypothesisTabComponent from "$lib/components/tabs/FloatingHypothesisTabComponent.svelte";
  import type { FloatingHypothesis } from "$lib/sharedState/model.svelte";
  import { invoke } from "@tauri-apps/api/core";

  export class FloatingHypothesisTab extends Tab {
    component = FloatingHypothesisTabComponent;

    #label: string;
    #floatingHypothesis: FloatingHypothesis = $state({ label: "", typecode: "", variable: "" });

    constructor(label: string) {
      super();
      this.#label = label;
    }

    async loadData(): Promise<void> {
      this.#floatingHypothesis = await invoke("get_floating_hypothesis_local", { label: this.#label });
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

    get floatingHypothesis() {
      return this.#floatingHypothesis;
    }
  }
</script>

<script lang="ts">
  import MetamathExpression from "../util/MetamathExpression.svelte";

  let { tab }: { tab: Tab } = $props();

  let floatingHypothesisTab = $derived.by(() => {
    if (tab instanceof FloatingHypothesisTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<div class="text-center">
  <div class="py-4">
    <h1 class="text-3xl">Floating Hypothesis: {floatingHypothesisTab.floatingHypothesis.label}</h1>
  </div>
  <div>
    <h2 class="text-xl">Statement:</h2>
    <MetamathExpression expression={floatingHypothesisTab.floatingHypothesis.typecode + " " + floatingHypothesisTab.floatingHypothesis.variable}></MetamathExpression>
  </div>
</div>
