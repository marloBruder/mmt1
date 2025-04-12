<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { variables }: { variables: string } = $props();
</script>

<ExplorerButton isOpenTab={false} explorerClick={() => {}} explorerDblClick={() => {}}>
  {"Variable" + (variables.includes(" ") ? "s" : "") + ": "}
  {#each variables.split(" ") as variable}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={variable}></MetamathExpression>
  {/each}
</ExplorerButton>
