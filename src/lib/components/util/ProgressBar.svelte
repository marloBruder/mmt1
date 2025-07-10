<script lang="ts">
  import { createInstanceId } from "$lib/sharedState/idGenerator.svelte";

  let { progress }: { progress: number } = $props();

  let progressBarId = "progress-bar-" + createInstanceId();

  $effect(() => {
    document.getElementById(progressBarId)!.style.width = progress + "%";
  });
</script>

<div class="mt-2 w-48 border rounded-lg p-2 relative z-0">
  <div class="z-10">Progress: {progress}{"%"}</div>
  <div id={progressBarId} class="bg-green-800 absolute start-0 top-0 rounded-lg h-full -z-10"></div>
</div>
