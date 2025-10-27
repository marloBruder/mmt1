<script lang="ts">
  import { goto } from "$app/navigation";
  import { version } from "$lib/sharedState/version.svelte";

  let backClick = () => {
    goto("/main");
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <div class="h-full w-full flex items-center justify-center">
    <div class="text-center">
      <div>You are using mmt1 v{version}</div>
      <div>mmt1 v1.0 was developed by Marlo Bruder</div>
      <div class="pt-6">Credit for some of the icons goes to <a href="https://feathericons.com" class="underline">https://feathericons.com</a></div>
      <div class="pt-6"><button class="underline" onclick={backClick}>Back</button></div>
    </div>
  </div>
</div>

<style>
  .custom-height-width-minus-margin {
    height: calc(100% - 1rem);
    width: calc(100% - 1rem);
  }
</style>
