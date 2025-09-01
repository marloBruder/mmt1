<script lang="ts">
  import { open } from "@tauri-apps/plugin-shell";
  import MetamathExpression from "./MetamathExpression.svelte";
  import TheoremLink from "./TheoremLink.svelte";
  import type { ParsedDescriptionSegment } from "$lib/sharedState/model.svelte";

  let { descriptionParsed }: { descriptionParsed: ParsedDescriptionSegment[] } = $props();

  let openLink = (url: string) => {
    open(url);
  };
</script>

<div class="text-left">
  {#each descriptionParsed as descriptionParsedSegement}
    {#if descriptionParsedSegement.discriminator == "DescriptionText"}
      {#each descriptionParsedSegement.text.split("\n") as paragraph, i}
        {#if i != 0}
          <span class="block leading-6"><br /></span>
        {/if}
        {paragraph}
      {/each}
    {:else if descriptionParsedSegement.discriminator == "DescriptionMathMode"}
      <MetamathExpression expression={descriptionParsedSegement.expression}></MetamathExpression>
    {:else if descriptionParsedSegement.discriminator == "DescriptionLabel"}
      <TheoremLink label={descriptionParsedSegement.label} theoremNumber={descriptionParsedSegement.theoremNumber}></TheoremLink>
    {:else if descriptionParsedSegement.discriminator == "DescriptionLink"}
      <button class="text-blue-400 underline" onclick={() => openLink(descriptionParsedSegement.url)}>{descriptionParsedSegement.url}</button>
    {:else if descriptionParsedSegement.discriminator == "DescriptionItalic"}
      <span class="italic">{descriptionParsedSegement.italic}</span>
    {:else if descriptionParsedSegement.discriminator == "DescriptionSubscript"}
      <span class="text-xs">{descriptionParsedSegement.subscript}</span>
    {:else if descriptionParsedSegement.discriminator == "DescriptionHtml"}
      <span class="prose prose-invert">{@html descriptionParsedSegement.html}</span>
    {:else if descriptionParsedSegement.discriminator == "DescriptionHtmlCharacterRef"}
      {@html "&" + descriptionParsedSegement.charRef + ";"}
    {/if}
  {/each}
</div>
