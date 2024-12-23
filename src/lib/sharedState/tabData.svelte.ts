import { invoke } from "@tauri-apps/api/core";
import type { InProgressTheorem, Theorem } from "./model.svelte";
import { nameListData } from "./nameListData.svelte";

class TabManager {
  #tabs: Tab[] = $state([]);

  #openedTabIndex: number = $state(0);

  async addTabAndOpen(newTab: Tab) {
    for (let [index, tab] of this.#tabs.entries()) {
      if (newTab.sameID(tab)) {
        this.#openedTabIndex = index;
        return;
      }
    }

    await newTab.loadData();
    this.#tabs.push(newTab);
    this.#openedTabIndex = this.#tabs.length - 1;
  }

  openTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      this.#openedTabIndex = tabIndex;
    }
  }

  // openEmptyTab() {
  //   this.#openedTabIndex = -1;
  // }

  closeTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      if (this.#openedTabIndex > tabIndex || (this.#openedTabIndex == tabIndex && tabIndex == this.#tabs.length - 1)) {
        this.#openedTabIndex--;
      }
      this.#tabs.splice(tabIndex, 1);
    }
  }

  closeCurrentTab() {
    this.closeTabWithIndex(this.#openedTabIndex);
  }

  // closeTab(tab: Tab) {
  //   for (let [index, otherTab] of this.#tabs.entries()) {
  //     if (tab.sameID(otherTab)) {
  //       this.closeTabWithIndex(index);
  //       return;
  //     }
  //   }
  // }

  getOpenedTab() {
    return this.#openedTabIndex >= 0 && this.#openedTabIndex < this.#tabs.length ? this.#tabs[this.#openedTabIndex] : null;
  }

  resetTabs() {
    this.#tabs = [];
  }

  get tabs() {
    return this.#tabs;
  }
}

let tabManager = new TabManager();

export { tabManager };

export abstract class Tab {
  abstract loadData(): Promise<void>;

  abstract name(): string;

  abstract sameID(tab: Tab): boolean;
}

export class TheoremTab extends Tab {
  #theoremName: string;
  #theorem: Theorem = $state({ name: "", description: "", disjoints: [], hypotheses: [], assertion: "", proof: null });

  constructor(theoremName: string) {
    super();
    this.#theoremName = theoremName;
  }

  async loadData(): Promise<void> {
    this.#theorem = await invoke("get_theorem_local", { name: this.#theoremName });
  }

  name(): string {
    return this.#theoremName;
  }

  sameID(tab: Tab): boolean {
    return tab instanceof TheoremTab && this.#theoremName == tab.theoremName;
  }

  get theorem() {
    return this.#theorem;
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
    if (this.#inProgressTheoremName != "") {
      this.#inProgressTheorem = await invoke("get_in_progress_theorem_local", { name: this.#inProgressTheoremName });
    }
  }

  name(): string {
    return this.#inProgressTheoremName !== "" ? this.#inProgressTheoremName : "New Tab";
  }

  sameID(tab: Tab): boolean {
    return tab instanceof EditorTab && this.#inProgressTheoremName == tab.inProgressTheoremName;
  }

  changeID(newID: string) {
    this.#inProgressTheoremName = newID;
  }

  async deleteTheorem() {
    await invoke("delete_in_progress_theorem", { name: this.#inProgressTheoremName });
    tabManager.closeCurrentTab();
    nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
    return;
  }

  async convertToTheorem() {
    let theorem: Theorem = await invoke("text_to_axium", { text: this.#inProgressTheorem.text });

    nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
    nameListData.addTheoremName(theorem.name);
    tabManager.closeCurrentTab();
    tabManager.addTabAndOpen(new TheoremTab(theorem.name));
  }

  get inProgressTheorem() {
    return this.#inProgressTheorem;
  }

  get inProgressTheoremName() {
    return this.#inProgressTheoremName;
  }
}

export class SettingsTab extends Tab {
  constructor() {
    super();
  }

  async loadData(): Promise<void> {}

  name(): string {
    return "Settings";
  }

  sameID(tab: Tab) {
    return tab instanceof SettingsTab;
  }

  validTab(): boolean {
    return true;
  }
}
