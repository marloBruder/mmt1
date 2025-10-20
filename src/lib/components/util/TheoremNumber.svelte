<script lang="ts" module>
  export let setupTheoremNumberStyleSheet = () => {
    let stylesheetText = "";

    for (let i = 0; i <= 325; i++) {
      stylesheetText =
        stylesheetText +
        `
.hsv${i} {
  color: hsl(${i}, 100%, 50%);
}`;
    }

    let stylesheet = document.createElement("style");
    stylesheet.textContent = stylesheetText;
    document.head.appendChild(stylesheet);
  };
</script>

<script lang="ts">
  import { globalState } from "$lib/sharedState/globalState.svelte";

  let { theoremNumber, normalTextSize = false }: { theoremNumber: number; normalTextSize?: boolean } = $props();

  let hsvCode = $derived.by(() => {
    if (globalState.databaseState !== null && globalState.databaseState.theoremAmount !== 0) {
      return Math.floor((theoremNumber * 325) / globalState.databaseState.theoremAmount);
    } else {
      return "";
    }
  });
</script>

<small class={(normalTextSize ? "text-base" : "text-xs ") + " hsv" + hsvCode + " "}>
  {theoremNumber}
</small>
