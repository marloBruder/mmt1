import { invoke } from "@tauri-apps/api/core";
import type { InProgressTheorem, Theorem } from "./model.svelte";
import { nameListData } from "./nameListData.svelte";
import { goto } from "$app/navigation";
import { page } from "$app/stores";
import { get } from "svelte/store";

class TabManager {
  #tabs: Tab[] = $state([]);

  async notifyTabOpened(newTab: Tab): Promise<Tab> {
    for (let tab of this.#tabs) {
      if (newTab.sameID(tab)) {
        return tab;
      }
    }

    await newTab.loadData();
    this.#tabs.push(newTab);
    return newTab;
  }

  openTabWithIndex(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      goto(this.#tabs[tabIndex].url());
    } else {
      goto("/main");
    }
  }

  closeTabWithIndex(tabIndex: number, navigate: boolean = true) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
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

  abstract url(): string;

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

  url(): string {
    return "/main/theorem/" + this.#theoremName;
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
    let theorem: Theorem = await invoke("text_to_axium", { text: this.#inProgressTheorem.text });

    nameListData.removeInProgressTheoremName(this.#inProgressTheoremName);
    nameListData.addTheoremName(theorem.name);
    tabManager.closeTabSameID(this, false);
    goto("/main/theorem/" + theorem.name);
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
