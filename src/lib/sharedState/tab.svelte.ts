import type { Component } from "svelte";
import type { DatabaseElementPageData } from "./model.svelte";

export abstract class Tab {
  abstract readonly component: Component<{ tab: Tab }>;
  readonly splitComponent: Component<{ pageData: DatabaseElementPageData | null }> | null = null;
  splitViewPageData: DatabaseElementPageData | null = $state(null);

  scrollTop: number = $state(0);
  splitViewScrollTop: number = $state(0);
  previousTab: Tab | null = null;
  nextTab: Tab | null = null;

  abstract loadData(): Promise<void>;

  abstract unloadData(): void;

  abstract name(): string;

  abstract sameTab(tab: Tab): boolean;

  async onTabOpen(): Promise<void> {}

  showUnsavedChanges(): boolean {
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
