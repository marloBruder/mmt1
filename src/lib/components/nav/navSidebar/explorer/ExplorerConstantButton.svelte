<script lang="ts">
  import MetamathExpression from "$lib/components/util/MetamathExpression.svelte";
  import ExplorerButton from "./ExplorerButton.svelte";

  let { constant }: { constant: string } = $props();
</script>

<ExplorerButton isOpenTab={false} explorerClick={() => {}} explorerDblClick={() => {}}>
  {"Constant: "}
  <MetamathExpression expression={constant}></MetamathExpression>
</ExplorerButton>
