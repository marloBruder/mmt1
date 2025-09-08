<script lang="ts">
  import ChevronUpIcon from "$lib/icons/arrows/ChevronUpIcon.svelte";
  import ChevronDownIcon from "$lib/icons/arrows/ChevronDownIcon.svelte";
  import type { Snippet } from "svelte";

  let { children, title, active }: { children: Snippet; title: string; active: boolean } = $props();

  let open: boolean = $state(false);

  let toggleOpen = () => {
    open = !open;
  };
</script>

<div class="w-full border-y">
  <button class="w-full text-left custom-bg-hover-color" onclick={toggleOpen}>
    <div class="flex flex-row">
      <div class={"flex-auto pl-1 " + (active ? " font-bold " : "")}>
        {title}
      </div>
      <div class="flex-initial w-6">
        {#if open}
          <ChevronDownIcon></ChevronDownIcon>
        {:else}
          <ChevronUpIcon></ChevronUpIcon>
        {/if}
      </div>
    </div>
  </button>
</div>
{#if open}
  <div class="w-full border-b">
    {@render children()}
  </div>
{/if}
