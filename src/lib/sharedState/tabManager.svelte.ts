import type { Component } from "svelte";
import type { DatabaseElementPageData } from "./model.svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type SplitTabState = "none" | "splitVertical" | "splitHorizontal" | "externalWindow";

class TabManager {
  #tabs: Tab[] = $state([]);
  #openTabIndex: number = $state(-1);
  #tempTabIndex: number = $state(-1);

  #splitTabState: SplitTabState = $state("none");

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
    await newTab.onTabOpen();
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
      await newTab.onTabOpen();
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
      this.#tabs[this.#openTabIndex].onTabOpen();
    } else {
      this.#openTabIndex = -1;
    }
  }

  async closeTabWithIndex(tabIndex: number) {
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

      await this.#tabs[this.#openTabIndex]?.onTabOpen();

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
      await openTab.previousTab.onTabOpen();

      this.#tabs[this.#openTabIndex] = openTab.previousTab;
    }
  }

  async switchToNextTab() {
    let openTab = this.getOpenTab();
    if (openTab && openTab.nextTab) {
      openTab.unloadData();
      await openTab.nextTab.loadData();
      await openTab.nextTab.onTabOpen();

      this.#tabs[this.#openTabIndex] = openTab.nextTab;
    }
  }

  closeOpenTab() {
    this.closeTabWithIndex(this.#openTabIndex);
  }

  closeTabsWithCondition(condition: (tab: Tab) => boolean) {
    for (let [i, tab] of this.#tabs.toReversed().entries()) {
      if (condition(tab)) {
        this.closeTabWithIndex(i);
      }
    }
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

  setSplitTabState(newState: SplitTabState) {
    if (this.#splitTabState !== "externalWindow" && newState === "externalWindow") {
      this.#splitTabState = newState;
      invoke("open_external_window");
    } else if (this.#splitTabState === "externalWindow" && newState !== "externalWindow") {
      // Set splitTabState before closing the window, so that the event "external-window-close" will be triggered after the state has been set.
      // For the "external-window-close" event listener, see below
      this.#splitTabState = newState;
      invoke("close_external_window");
    } else {
      this.#splitTabState = newState;
    }
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

  get splitTabState() {
    return this.#splitTabState;
  }
}

let tabManager = new TabManager();
export { tabManager };

export abstract class Tab {
  abstract readonly component: Component<{ tab: Tab }>;
  readonly splitComponent: Component<{ pageData: DatabaseElementPageData | null }> | null = null;
  splitViewPageData: DatabaseElementPageData | null = $state(null);

  scrollTop: number = 0;
  previousTab: Tab | null = null;
  nextTab: Tab | null = null;

  abstract loadData(): Promise<void>;

  abstract unloadData(): void;

  abstract name(): string;

  abstract sameTab(tab: Tab): boolean;

  async onTabOpen(): Promise<void> {}

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

  async format(): Promise<void> {}

  formatDisabled(): boolean {
    return true;
  }

  async renumber(): Promise<void> {}

  renumberDisabled(): boolean {
    return true;
  }

  async addToDatabase(): Promise<void> {}

  addToDatabaseDisabled(): boolean {
    return true;
  }
}

listen("external-window-close", () => {
  if (tabManager.splitTabState === "externalWindow") {
    tabManager.setSplitTabState("none");
  }
});
