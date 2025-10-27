<script lang="ts">
  import { goto } from "$app/navigation";

  let backClick = () => {
    goto("/main");
  };
</script>

<div class="custom-height-width-minus-margin m-2 rounded-lg custom-bg-color overflow-hidden">
  <div class="h-full w-full flex items-center justify-center">
    <div class="text-center">
      <div>For instructions on how to use mmt1 please see <a href="https://github.com/marloBruder/mmt1/tree/release/guides" class="underline">https://github.com/marloBruder/mmt1/tree/release/guides</a></div>
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
