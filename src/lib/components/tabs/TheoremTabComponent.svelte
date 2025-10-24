<script lang="ts" module>
  import TheoremTabComponent from "$lib/components/tabs/TheoremTabComponent.svelte";
  import type { TheoremPageData } from "$lib/sharedState/model.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let theoremPageDataDefault: TheoremPageData = {
    theorem: {
      label: "",
      description: "",
      tempVariables: [],
      tempFloatingHypotheses: [],
      distincts: [],
      hypotheses: [],
      assertion: "",
      proof: null,
    },
    theoremNumber: 0,
    proofLines: [],
    lastTheoremLabel: null,
    nextTheoremLabel: null,
    previewErrors: [],
    previewDeletedMarkers: [],
    previewConfirmations: [],
    previewConfirmationsRecursive: [],
    previewUnifyMarkers: [],
    axiomDependencies: [],
    definitionDependencies: [],
    references: [],
    descriptionParsed: [],
    invalidHtml: false,
    discriminator: "TheoremPageData",
    proofIncomplete: false,
  };

  export class TheoremTab extends Tab {
    component = TheoremTabComponent;

    #theoremLabel: string;
    #pageData: TheoremPageData = $state(theoremPageDataDefault);
    showAll: boolean = $state(false);

    constructor(theoremLabel: string) {
      super();
      this.#theoremLabel = theoremLabel;
      this.showAll = settingsData.settings.defaultShowAll;
    }

    async loadData(): Promise<void> {
      this.#pageData = await invoke("get_theorem_page_data", {
        label: this.#theoremLabel,
        showAll: this.showAll,
      });
    }

    unloadData(): void {
      this.#pageData = theoremPageDataDefault;
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
  import TheoremPage from "../pages/TheoremPage.svelte";
  import { settingsData } from "$lib/sharedState/settingsData.svelte";
  import { Tab } from "$lib/sharedState/tab.svelte";

  let { tab }: { tab: Tab } = $props();

  let theoremTab = $derived.by(() => {
    if (tab instanceof TheoremTab) {
      return tab;
    }
    throw Error("Wrong Tab Type:" + typeof tab + ", " + JSON.stringify(tab));
  });

  let pageData = $derived(theoremTab.pageData);
</script>

<TheoremPage {pageData} {theoremTab}></TheoremPage>
