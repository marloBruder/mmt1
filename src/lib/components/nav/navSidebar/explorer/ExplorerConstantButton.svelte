<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { constants }: { constants: string } = $props();
</script>

<ExplorerButton isOpenTab={false} explorerClick={() => {}} explorerDblClick={() => {}}>
  {"Constant" + (constants.includes(" ") ? "s" : "") + ": "}
  {#each constants.split(" ") as constant}
    <div class="inline-block w-1"></div>
    <MetamathExpression expression={constant}></MetamathExpression>
  {/each}
</ExplorerButton>
