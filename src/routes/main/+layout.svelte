<script lang="ts">
  import TitleBar from "$lib/components/titleBar/TitleBar.svelte";
  import HorizontalSplit from "$lib/components/util/HorizontalSplit.svelte";
</script>

<div class="h-screen w-screen fixed custom-bg-bg-color text-gray-300">
  <HorizontalSplit>
    {#snippet first()}
      <div class="h-8 w-full overflow-hidden">
        <TitleBar></TitleBar>
      </div>
    {/snippet}
    {#snippet second()}
      <slot />
    {/snippet}
  </HorizontalSplit>
</div>
