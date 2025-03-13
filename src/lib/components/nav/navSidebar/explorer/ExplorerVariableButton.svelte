<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { variable }: { variable: string } = $props();
</script>

<ExplorerButton isOpenTab={false} explorerClick={() => {}} explorerDblClick={() => {}}>
  {"Variable: "}
  <MetamathExpression expression={variable}></MetamathExpression>
</ExplorerButton>
