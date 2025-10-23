<script lang="ts" module>
  import type { HeaderPageData, HeaderPath } from "$lib/sharedState/model.svelte";
  import HeaderTabComponent from "$lib/components/tabs/HeaderTabComponent.svelte";

  const defaultHeaderPageData: HeaderPageData = {
    title: "",
    descriptionParsed: [],
    headerPath: "",
    discriminator: "HeaderPageData",
  };

  export class HeaderTab extends Tab {
    component = HeaderTabComponent;

    #headerPath: HeaderPath;
    #pageData: HeaderPageData = $state(defaultHeaderPageData);

    constructor(headerPath: HeaderPath) {
      super();
      this.#headerPath = headerPath;
    }

    async loadData(): Promise<void> {
      this.#pageData = (await invoke("get_header_page_data", { headerPath: this.#headerPath })) as HeaderPageData;
    }

    unloadData(): void {
      this.#pageData = defaultHeaderPageData;
    }

    name(): string {
      return "Header " + util.headerPathToStringRep(this.#headerPath);
    }

    sameTab(tab: Tab): boolean {
      return tab instanceof HeaderTab && tab.headerPath.path == this.#headerPath.path;
    }

    get headerPath() {
      return this.#headerPath;
    }

    get pageData() {
      return this.#pageData;
    }
  }
</script>

<script lang="ts">
  import { Tab } from "$lib/sharedState/tab.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { util } from "$lib/sharedState/util.svelte";
  import HeaderPage from "../pages/HeaderPage.svelte";

  let { tab }: { tab: Tab } = $props();

  let headerTab: HeaderTab = $derived.by(() => {
    if (tab instanceof HeaderTab) {
      return tab;
    }
    throw Error("Wrong Tab Type!");
  });
</script>

<HeaderPage pageData={headerTab.pageData}></HeaderPage>
