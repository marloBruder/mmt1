import type { PageLoad } from "./$types";
import editorTabs from "$lib/sharedState/mainData.svelte";

export const load: PageLoad = ({ params }) => {
  return {
    tabID: Number.parseInt(params.tabID),
  };
};
