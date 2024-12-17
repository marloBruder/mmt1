import { inProgressTheoremData } from "./mainData.svelte";

class TabManager {
  #tabs: Tab[] = $state([]);

  #openedTabIndex: number = $state(-1);

  addTabAndOpen(newTab: Tab) {
    if (!newTab.validTab()) {
      return;
    }

    for (let [index, tab] of this.#tabs.entries()) {
      if (newTab.sameTab(tab)) {
        this.#openedTabIndex = index;
        return;
      }
    }

    this.#tabs.push(newTab);
    this.#openedTabIndex = this.#tabs.length - 1;
  }

  getOpenedTab() {
    return this.#openedTabIndex != -1 ? this.#tabs[this.#openedTabIndex] : null;
  }

  openTab(tabIndex: number) {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      this.#openedTabIndex = tabIndex;
    }
  }

  openEmptyTab() {
    this.#openedTabIndex = -1;
  }

  get tabs() {
    return this.#tabs;
  }
}

export abstract class Tab {
  abstract name(): string;

  abstract sameTab(tab: Tab): boolean;

  abstract validTab(): boolean;
}

export class TheoremTabClass extends Tab {
  #theoremName: string;

  constructor(theoremName: string) {
    super();
    this.#theoremName = theoremName;
  }

  name(): string {
    return this.#theoremName;
  }

  sameTab(tab: Tab): boolean {
    return tab instanceof TheoremTabClass && this.#theoremName == tab.theoremName;
  }

  validTab(): boolean {
    return true;
  }

  get theoremName() {
    return this.#theoremName;
  }
}

export class EditorTabClass extends Tab {
  #localID: number;

  constructor(localID: number) {
    super();
    this.#localID = localID;
  }

  name(): string {
    let theorem = inProgressTheoremData.getTheoremByID(this.#localID);
    return theorem ? theorem.name : "";
  }

  sameTab(tab: Tab): boolean {
    return tab instanceof EditorTabClass && this.#localID == tab.localID;
  }

  validTab(): boolean {
    return inProgressTheoremData.validID(this.#localID);
  }

  get localID() {
    return this.#localID;
  }
}

let tabManager = new TabManager();

export { tabManager };
