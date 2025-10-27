<script lang="ts">
  import { goto } from "$app/navigation";
  import RoundButton from "$lib/components/util/RoundButton.svelte";
  import { version } from "$lib/sharedState/version.svelte";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { check, Update } from "@tauri-apps/plugin-updater";

  let updateStatus: "notChecked" | "noUpdates" | "updates" = $state("notChecked");

  let update: Update | null = $state(null);

  let disabled = $state(false);

  let onCheckForUpdatesClick = async () => {
    let newUpdate = await check();
    if (newUpdate) {
      updateStatus = "updates";
      update = newUpdate;
    } else {
      updateStatus = "noUpdates";
    }
  };

  let onUpdateClick = async () => {
    disabled = true;
    await update!.downloadAndInstall();
    await relaunch();
  };

  let backClick = () => {
    goto("/main");
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <div class="h-full w-full flex items-center justify-center">
    <div class="text-center">
      <div class="py-4">You are currently using mmt1 v{version}</div>
      {#if updateStatus === "noUpdates"}
        <div class="py-4">No updates found.</div>
      {:else if updateStatus === "updates"}
        <div class="py-4">
          <div>Update found.</div>
          <div><RoundButton onclick={onUpdateClick} {disabled}>Update to mmt1 v{update!.version}</RoundButton></div>
        </div>
      {/if}
      <div class="py-4"><RoundButton onclick={onCheckForUpdatesClick} {disabled}>Check for Updates</RoundButton></div>
      <div class="py-4"><button class="underline" onclick={backClick} {disabled}>Back</button></div>
    </div>
  </div>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
