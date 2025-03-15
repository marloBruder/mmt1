<script lang="ts">
  import { SearchTab } from "$lib/components/tabs/SearchTabComponent.svelte";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import type { SearchParameters } from "$lib/sharedState/model.svelte";
  import { tabManager } from "$lib/sharedState/tabManager.svelte";

  let searchParameters: SearchParameters = $state({ label: "", start: 0, amount: 100 });

  let searchClick = async () => {
    tabManager.openTab(new SearchTab(searchParameters), true);
  };
</script>

<div class="p-2">
  <div>
    <label for="search-input">Label:</label>
    <br />
    <input id="search-input" class="border border-black rounded" bind:value={searchParameters.label} autocomplete="off" />
  </div>
  <div class="pt-2">
    <RoundButton onclick={searchClick}>Search</RoundButton>
  </div>
</div>
