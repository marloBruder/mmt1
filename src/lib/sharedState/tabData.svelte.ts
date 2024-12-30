import { invoke } from "@tauri-apps/api/core";
import type { Constant, FloatingHypotheses, InProgressTheorem, Theorem, TheoremPageData, Variable } from "./model.svelte";
import { nameListData } from "./nameListData.svelte";
import { goto } from "$app/navigation";
import { page } from "$app/stores";
import { get } from "svelte/store";

class TabManager {
  #tabs: Tab[] = $state([]);
  #activeTabIndex: number = $state(-1);

  async notifyTabOpened(newTab: Tab): Promise<Tab> {
    for (let [index, tab] of this.#tabs.entries()) {
      if (newTab.sameID(tab)) {
        this.#activeTabIndex = index;
        return tab;
      }
    }

    await newTab.loadData();
    this.#tabs.push(newTab);
    this.#activeTabIndex = this.#tabs.length - 1;
    return newTab;
  }

  openTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      this.#activeTabIndex = tabIndex;
      goto(this.#tabs[tabIndex].url());
    } else {
      this.#activeTabIndex = -1;
      goto("/main");
    }
  }

  closeTabWithIndex(tabIndex: number, navigate: boolean = true) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      if (tabIndex < this.#activeTabIndex) {
        this.#activeTabIndex--;
      }

      let closedCurrentTab = false;
      if (this.#tabs[tabIndex].url() === get(page).url.pathname) {
        closedCurrentTab = true;
      }

      this.#tabs.splice(tabIndex, 1);

      if (closedCurrentTab && navigate) {
        let newTabIndex = tabIndex;
        if (newTabIndex === this.#tabs.length) {
          newTabIndex--;
        }
        this.openTabWithIndex(newTabIndex);
      }
    }
  }

  closeTabSameID(tab: Tab, navigate: boolean = true) {
    for (let [index, otherTab] of this.#tabs.entries()) {
      if (tab.sameID(otherTab)) {
        this.closeTabWithIndex(index, navigate);
        return;
      }
    }
  }

  resetTabs() {
    this.#tabs = [];
    this.#activeTabIndex = -1;
  }

  get tabs() {
    return this.#tabs;
  }

  get activeTabIndex() {
    return this.#activeTabIndex;
  }
}

let tabManager = new TabManager();
export { tabManager };

export abstract class Tab {
  abstract loadData(): Promise<void>;

  abstract name(): string;

  abstract url(): string;

  abstract sameID(tab: Tab): boolean;
}

export class TheoremTab extends Tab {
  #theoremName: string;
  #pageData: TheoremPageData = $state({ theorem: { name: "", description: "", disjoints: [], hypotheses: [], assertion: "", proof: null }, proofLines: [] });

  constructor(theoremName: string) {
    super();
    this.#theoremName = theoremName;
  }

  async loadData(): Promise<void> {
    this.#pageData = await invoke("get_theorem_page_data_local", { name: this.#theoremName });
    console.log(this.#pageData);
  }

  name(): string {
    return this.#theoremName;
  }

  url(): string {
    return "/main/theorem/" + this.#theoremName;
  }

  sameID(tab: Tab): boolean {
    return tab instanceof TheoremTab && this.#theoremName == tab.theoremName;
  }

  get pageData() {
    return this.#pageData;
  }

  get theoremName() {
    return this.#theoremName;
  }
}

export class EditorTab extends Tab {
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

  url(): string {
    return "/main/editor/" + this.#inProgressTheoremName;
  }

  sameID(tab: Tab): boolean {
    return tab instanceof EditorTab && this.#inProgressTheoremName == tab.inProgressTheoremName;
  }

  changeEditorID(newID: string) {
    this.#inProgressTheoremName = newID;
  }

  async deleteTheorem() {
    await invoke("delete_in_progress_theorem", { name: this.#inProgressTheoremName });
    tabManager.closeTabSameID(this);
    nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
    return;
  }

  async convertToTheorem() {
    await invoke("turn_into_theorem", { inProgressTheorem: this.#inProgressTheorem });

    nameListData.removeInProgressTheoremName(this.#inProgressTheorem.name);
    nameListData.addTheoremName(this.#inProgressTheorem.name);
    tabManager.closeTabSameID(this, false);
    goto("/main/theorem/" + this.#inProgressTheorem.name);
  }

  get inProgressTheorem() {
    return this.#inProgressTheorem;
  }

  get inProgressTheoremName() {
    return this.#inProgressTheoremName;
  }
}

export class SettingsTab extends Tab {
  constants: Constant[] = $state([]);
  variables: Variable[] = $state([]);
  floatingHypotheses: FloatingHypotheses[] = $state([]);

  constructor() {
    super();
  }

  async loadData(): Promise<void> {
    this.constants = await invoke("get_constants_local");
    this.variables = await invoke("get_variables_local");
    this.floatingHypotheses = await invoke("get_floating_hypotheses_local");
  }

  name(): string {
    return "Settings";
  }

  url(): string {
    return "/main/settings";
  }

  sameID(tab: Tab) {
    return tab instanceof SettingsTab;
  }

  validTab(): boolean {
    return true;
  }
}
