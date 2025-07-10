<script>
  import ProgressBar from "$lib/components/util/ProgressBar.svelte";
  import { globalState } from "$lib/sharedState/globalState.svelte";
</script>

<div class="p-2">
  <div>
    Progress calculating parse trees:
    <ProgressBar progress={globalState.grammarCalculationsProgress}></ProgressBar>
  </div>
</div>
