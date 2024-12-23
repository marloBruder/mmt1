<script lang="ts">
  import RoundButton from "$lib/components/util/RoundButton.svelte";

  let editing = $state(false);

  let editVariables = () => {
    editing = true;
  };
  let saveVariables = () => {
    editing = false;
  };
</script>

<div class="p-2">
  <div class="text-2xl pb-4">
    <h1>Variables:</h1>
  </div>
  <div class="pb-2">
    <RoundButton onclick={editVariables} disabled={editing}>Edit</RoundButton>
    <RoundButton onclick={saveVariables} disabled={!editing}>Save</RoundButton>
  </div>
  {#if !editing}
    <div class="pl-4">
      <ol class="list-decimal">
        <!-- {#each variableData.variables as variable}
          <li>{variable.type + " " + variable.name}</li>
        {/each} -->
      </ol>
    </div>
  {:else}
    <div>
      <textarea class="w-full h-96"></textarea>
    </div>
  {/if}
</div>
