import { tabManager, TheoremTab } from "$lib/sharedState/tabData.svelte";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ params, url }) => {
  console.log("Loading: " + url.pathname);
  let tab = await tabManager.notifyTabOpened(new TheoremTab(params.theoremName));
  if (!(tab instanceof TheoremTab)) {
    throw new Error("TabManager returned wrong Tab Type");
  }
  return { tab };
};
