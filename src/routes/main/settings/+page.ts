import { SettingsTab, tabManager } from "$lib/sharedState/tabData.svelte";
import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  let tab = await tabManager.notifyTabOpened(new SettingsTab());
  if (!(tab instanceof SettingsTab)) {
    throw new Error("TabManager returned wrong Tab Type");
  }
  return { tab };
};
