import inProgressTheoremData from "./mainData.svelte";

class TabManager {
  #tabs: Tab[] = $state([]);

  #openedTabIndex: number = $state(-1);

  addTabAndOpen = (newTab: Tab) => {
    if (newTab instanceof EditorTabClass) {
      for (let [index, tab] of this.#tabs.entries()) {
        if (tab instanceof EditorTabClass && tab.localID == newTab.localID) {
          this.#openedTabIndex = index;
          return;
        }
      }
    }
    this.#tabs.push(newTab);
    this.#openedTabIndex = this.#tabs.length - 1;
  };

  getOpenedTab = () => {
    return this.#openedTabIndex != -1 ? this.#tabs[this.#openedTabIndex] : null;
  };

  openTab = (tabIndex: number) => {
    if (tabIndex >= 0 && tabIndex < this.#tabs.length) {
      this.#openedTabIndex = tabIndex;
    }
  };

  get tabs() {
    return this.#tabs;
  }
}

export abstract class Tab {
  abstract name(): string;
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

  get localID() {
    return this.#localID;
  }
}

export default new TabManager();
