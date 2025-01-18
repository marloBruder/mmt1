<script lang="ts">
  import { goto } from "$app/navigation";

  let { theoremName, openTheoremName }: { theoremName: string; openTheoremName: string | null } = $props();

  let explorerClick = (name: string) => {
    goto("/main/theorem/" + name);
  };
</script>

<div>
  <button class={"w-full text-left pl-2 " + (theoremName === openTheoremName ? " bg-gray-300 " : " hover:bg-gray-200 ")} onclick={() => explorerClick(theoremName)}>{theoremName}</button>
</div>
