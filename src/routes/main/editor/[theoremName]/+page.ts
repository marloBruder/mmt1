import { EditorTab, tabManager } from "$lib/sharedState/tabData.svelte";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params }) => {
  let tab = await tabManager.notifyTabOpened(new EditorTab(params.theoremName));
  if (!(tab instanceof EditorTab)) {
    throw new Error("TabManager returned wrong Tab Type");
  }
  return { tab };
};
