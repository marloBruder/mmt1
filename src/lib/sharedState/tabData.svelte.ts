import { invoke } from "@tauri-apps/api/core";
import type { Constant, FloatingHypotheses, HtmlRepresentation, InProgressTheorem, SearchParameters, TheoremListEntry, TheoremPageData, TheoremPath, Variable } from "./model.svelte";
import { nameListData } from "./nameListData.svelte";
import { explorerData } from "./explorerData.svelte";
import type { Component } from "svelte";
import TheoremTabComponent from "$lib/components/tabs/TheoremTabComponent.svelte";
import SettingsTabComponent from "$lib/components/tabs/SettingsTabComponent.svelte";
import EditorTabComponent from "$lib/components/tabs/EditorTabComponent.svelte";
import TheoremExplorerTabComponent from "$lib/components/tabs/TheoremExplorerTabComponent.svelte";
import SearchTabComponent from "$lib/components/tabs/SearchTabComponent.svelte";

class TabManager {
  #tabs: Tab[] = $state([]);
  #openTabIndex: number = $state(-1);
  #tempTabIndex: number = $state(-1);

  getOpenTab(): Tab | null {
    return 0 <= this.#openTabIndex && this.#openTabIndex < this.#tabs.length ? this.#tabs[this.#openTabIndex] : null;
  }

  async openTab(newTab: Tab) {
    for (let [index, tab] of this.#tabs.entries()) {
      if (tab.sameTab(newTab)) {
        this.#openTabIndex = index;
        return;
      }
    }

    await newTab.loadData();
    if (0 <= this.#tempTabIndex && this.#tempTabIndex < this.#tabs.length) {
      this.#tabs[this.#tempTabIndex] = newTab;
      this.#openTabIndex = this.#tempTabIndex;
    } else {
      this.#tabs.push(newTab);
      this.#openTabIndex = this.#tabs.length - 1;
      this.#tempTabIndex = this.#tabs.length - 1;
    }
  }

  async changeTab(newTab: Tab) {
    if (this.#tabs.length == 0) {
      this.openTab(newTab);
    } else {
      await newTab.loadData();
      this.#tabs[this.#openTabIndex] = newTab;
    }
  }

  makeOpenTempTabPermanent() {
    this.#tempTabIndex = -1;
  }

  makeTempTabWithIndexPermanent(index: number) {
    if (this.#tempTabIndex == index) {
      this.#tempTabIndex = -1;
    }
  }

  makeSameTempTabPermanent(tab: Tab) {
    if (0 <= this.#tempTabIndex && this.#tempTabIndex < this.#tabs.length && this.#tabs[this.#tempTabIndex].sameTab(tab)) {
      this.#tempTabIndex = -1;
    }
  }

  // async openTab(newTab: Tab) {
  //   for (let [index, tab] of this.#tabs.entries()) {
  //     if (tab.sameTab(newTab)) {
  //       this.#openTabIndex = index;
  //       return;
  //     }
  //   }

  //   await newTab.loadData();
  //   this.#tabs.push(newTab);
  //   this.#openTabIndex = this.#tabs.length - 1;
  // }

  // async notifyTabOpened(newTab: Tab): Promise<Tab> {
  //   for (let [index, tab] of this.#tabs.entries()) {
  //     if (newTab.sameID(tab)) {
  //       this.#openTabIndex = index;
  //       return tab;
  //     }
  //   }

  //   await newTab.loadData();
  //   this.#tabs.push(newTab);
  //   this.#openTabIndex = this.#tabs.length - 1;
  //   return newTab;
  // }

  openTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      this.#openTabIndex = tabIndex;
      // goto(this.#tabs[tabIndex].url());
    } else {
      this.#openTabIndex = -1;
      // goto("/main");
    }
  }

  closeTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      if (tabIndex < this.#openTabIndex || (tabIndex == this.#openTabIndex && tabIndex == this.#tabs.length - 1)) {
        this.#openTabIndex--;
      }

      if (tabIndex < this.#tempTabIndex) {
        this.#tempTabIndex--;
      } else if (tabIndex == this.#tempTabIndex) {
        this.#tempTabIndex = -1;
      }
      // let closedCurrentTab = false;
      // if (this.#tabs[tabIndex].url() === get(page).url.pathname) {
      //   closedCurrentTab = true;
      // }

      this.#tabs.splice(tabIndex, 1);

      // if (closedCurrentTab && navigate) {
      //   let newTabIndex = tabIndex;
      //   if (newTabIndex === this.#tabs.length) {
      //     newTabIndex--;
      //   }
      //   this.openTabWithIndex(newTabIndex);
      // }
    }
  }

  // closeTabSameID(tab: Tab, navigate: boolean = true) {
  //   for (let [index, otherTab] of this.#tabs.entries()) {
  //     if (tab.sameID(otherTab)) {
  //       this.closeTabWithIndex(index, navigate);
  //       return;
  //     }
  //   }
  // }

  closeOpenTab() {
    this.closeTabWithIndex(this.#openTabIndex);
  }

  resetTabs() {
    this.#tabs = [];
    this.#openTabIndex = -1;
    this.#tempTabIndex = -1;
  }

  get tabs() {
    return this.#tabs;
  }

  get openTabIndex() {
    return this.#openTabIndex;
  }

  get tempTabIndex() {
    return this.#tempTabIndex;
  }
}

let tabManager = new TabManager();
export { tabManager };

export abstract class Tab {
  abstract readonly component: Component<{ tab: Tab }>;

  abstract loadData(): Promise<void>;

  abstract name(): string;

  abstract sameTab(tab: Tab): boolean;
}

export class TheoremTab extends Tab {
  component = TheoremTabComponent;

  #theoremName: string;
  #pageData: TheoremPageData = $state({ theorem: { name: "", description: "", disjoints: [], hypotheses: [], assertion: "", proof: null }, theoremNumber: 0, proofLines: [] });

  constructor(theoremName: string) {
    super();
    this.#theoremName = theoremName;
  }

  async loadData(): Promise<void> {
    this.#pageData = await invoke("get_theorem_page_data_local", { name: this.#theoremName });
  }

  name(): string {
    return this.#theoremName;
  }

  sameTab(tab: Tab): boolean {
    return tab instanceof TheoremTab && this.#theoremName == tab.theoremName;
  }

  get pageData() {
    return this.#pageData;
  }

  get theoremName() {
    return this.#theoremName;
  }
}

export class TheoremExplorerTab extends Tab {
  component = TheoremExplorerTabComponent;

  #start: number = $state(1);
  #theoremList: TheoremListEntry[] = $state([]);

  async loadData(): Promise<void> {
    this.#theoremList = await invoke("get_theorem_list_local", { from: this.#start, to: this.#start + 100 });
  }

  async changePage(newStart: number) {
    this.#start = newStart;
    await this.loadData();
  }

  name(): string {
    return "Theorem Explorer";
  }

  sameTab(tab: Tab): boolean {
    return tab instanceof TheoremExplorerTab;
  }

  get start() {
    return this.#start;
  }

  get theoremList() {
    return this.#theoremList;
  }
}

export class SearchTab extends Tab {
  component = SearchTabComponent;

  #searchParameters: SearchParameters = $state({ label: "" });
  #searchResult: TheoremListEntry[] = $state([]);

  constructor(searchParameters: SearchParameters) {
    super();
    this.#searchParameters.label = searchParameters.label;
  }

  async loadData(): Promise<void> {
    this.#searchResult = await invoke("search_theorems", { searchParameters: this.#searchParameters });
  }

  name(): string {
    return "Search: " + this.#searchParameters.label;
  }

  sameTab(tab: Tab): boolean {
    return false;
  }

  get searchParameters() {
    return this.#searchParameters;
  }

  get searchResult() {
    return this.#searchResult;
  }
}

export class EditorTab extends Tab {
  component = EditorTabComponent;

  #inProgressTheoremName: string = $state("");
  #inProgressTheorem: InProgressTheorem = $state({ name: "", text: "" });

  constructor(inProgressTheoremName: string) {
    super();
    this.#inProgressTheoremName = inProgressTheoremName;
  }

  async loadData(): Promise<void> {
    this.#inProgressTheorem = await invoke("get_in_progress_theorem_local", { name: this.#inProgressTheoremName });
  }

  name(): string {
    return this.#inProgressTheoremName;
  }

  sameTab(tab: Tab): boolean {
    return tab instanceof EditorTab && this.#inProgressTheoremName == tab.inProgressTheoremName;
  }

  changeEditorID(newID: string) {
    this.#inProgressTheoremName = newID;
  }

  async deleteTheorem() {
    await invoke("delete_in_progress_theorem", { name: this.#inProgressTheoremName });
    tabManager.closeOpenTab();
    nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
    return;
  }

  async convertToTheorem(placeAfter: string) {
    let dataUnknown = await invoke("turn_into_theorem", { inProgressTheorem: this.#inProgressTheorem, positionName: placeAfter });
    let theoremPath = dataUnknown as TheoremPath;

    nameListData.removeInProgressTheoremName(this.#inProgressTheorem.name);
    await explorerData.addTheoremName(theoremPath, this.#inProgressTheorem.name);
    tabManager.changeTab(new TheoremTab(this.#inProgressTheorem.name));
  }

  get inProgressTheorem() {
    return this.#inProgressTheorem;
  }

  get inProgressTheoremName() {
    return this.#inProgressTheoremName;
  }
}

export class SettingsTab extends Tab {
  component = SettingsTabComponent;

  constants: Constant[] = $state([]);
  variables: Variable[] = $state([]);
  floatingHypotheses: FloatingHypotheses[] = $state([]);
  htmlRepresentations: HtmlRepresentation[] = $state([]);

  constructor() {
    super();
  }

  async loadData(): Promise<void> {
    this.constants = await invoke("get_constants_local");
    this.variables = await invoke("get_variables_local");
    this.floatingHypotheses = await invoke("get_floating_hypotheses_local");
    this.htmlRepresentations = await invoke("get_html_representations_local");
  }

  name(): string {
    return "Settings";
  }

  sameTab(tab: Tab) {
    return tab instanceof SettingsTab;
  }

  validTab(): boolean {
    return true;
  }
}
