import type { Component } from "svelte";

class TabManager {
  #tabs: Tab[] = $state([]);
  #openTabIndex: number = $state(-1);
  #tempTabIndex: number = $state(-1);

  getOpenTab(): Tab | null {
    return 0 <= this.#openTabIndex && this.#openTabIndex < this.#tabs.length ? this.#tabs[this.#openTabIndex] : null;
  }

  async openTab(newTab: Tab, permanent: boolean = false) {
    for (let [index, tab] of this.#tabs.entries()) {
      if (tab.sameTab(newTab)) {
        this.#openTabIndex = index;
        return;
      }
    }

    await newTab.loadData();
    if (0 <= this.#tempTabIndex && this.#tempTabIndex < this.#tabs.length) {
      this.#tabs[this.#tempTabIndex].unloadData();
      this.#tabs[this.#tempTabIndex] = newTab;
      this.#openTabIndex = this.#tempTabIndex;
      if (permanent) {
        this.#tempTabIndex = -1;
      }
    } else {
      this.#tabs.push(newTab);
      this.#openTabIndex = this.#tabs.length - 1;
      if (!permanent) {
        this.#tempTabIndex = this.#tabs.length - 1;
      }
    }
  }

  async changeTab(newTab: Tab) {
    if (this.getOpenTab() === null) {
      this.openTab(newTab);
    } else {
      await newTab.loadData();
      this.#tabs[this.#openTabIndex].unloadData();

      newTab.previousTab = this.#tabs[this.#openTabIndex];
      this.#tabs[this.#openTabIndex].nextTab = newTab;
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

      this.#tabs[tabIndex].unloadData();
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

  async switchToPreviousTab() {
    let openTab = this.getOpenTab();
    if (openTab && openTab.previousTab) {
      openTab.unloadData();
      await openTab.previousTab.loadData();

      this.#tabs[this.#openTabIndex] = openTab.previousTab;
    }
  }

  async switchToNextTab() {
    let openTab = this.getOpenTab();
    if (openTab && openTab.nextTab) {
      openTab.unloadData();
      await openTab.nextTab.loadData();

      this.#tabs[this.#openTabIndex] = openTab.nextTab;
    }
  }

  closeOpenTab() {
    this.closeTabWithIndex(this.#openTabIndex);
  }

  resetTabs() {
    this.#tabs = [];
    this.#openTabIndex = -1;
    this.#tempTabIndex = -1;
  }

  isSameTabOpen(tab: Tab) {
    let openTab = this.#tabs[this.#openTabIndex];
    if (openTab) {
      return tab.sameTab(openTab);
    }
    return false;
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

  scrollTop: number = 0;
  previousTab: Tab | null = null;
  nextTab: Tab | null = null;

  abstract loadData(): Promise<void>;

  abstract unloadData(): void;

  abstract name(): string;

  abstract sameTab(tab: Tab): boolean;

  showDot(): boolean {
    return false;
  }

  async saveFile(): Promise<void> {}

  saveFileDisabled(): boolean {
    return true;
  }

  async unify(): Promise<void> {}

  unifyDisabled(): boolean {
    return true;
  }

  async addToDatabase(): Promise<void> {}

  addToDatabaseDisabled(): boolean {
    return true;
  }
}
