<script>
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
</script>

<div class="p-2">
  <div>
    Progress calculating parse trees:
    <ProgressBar progress={globalState.databaseState.grammarCalculationsProgress}></ProgressBar>
  </div>
</div>
