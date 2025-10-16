<script lang="ts">
  import RoundButton from "./RoundButton.svelte";

  let { invalidHtml, htmlType }: { invalidHtml: [string, string][]; htmlType: "symbol" | "theoremDescription" | "headerDescription" } = $props();

  let page: number = $state(0);

  let invalidHtmlPreviousPage = () => {
    page -= 1;
  };

  let invalidHtmlNextPage = () => {
    page += 1;
  };
</script>

{#if invalidHtml.length != 0}
  <div class="mt-2 p-2 mx-12 border rounded-lg">
    <h2 class="text-red-600">WARNING</h2>
    {#if htmlType === "symbol"}
      The HTML representation of symbols in this database does not follow all rules for safe HTML checked by mmt1.
    {:else if htmlType === "theoremDescription"}
      The HTML found in the description of theorems does follow all rules for safe HTML checked by mmt1.
    {:else if htmlType === "headerDescription"}
      The HTML found in the description of headers does follow all rules for safe HTML checked by mmt1.
    {/if}
    The following
    <span class="text-red-600">{invalidHtml.length}</span>
    HTML
    {#if htmlType === "symbol"}
      representations
    {:else}
      snippets
    {/if}
    may be dangerous. This does not mean that they must be dangerous, but that they could be. Please manually check that
    <span class="text-red-600">EVERY SINGLE</span>
    one of them is safe:
    <div class="mt-4">
      <table class=" mx-auto">
        <thead>
          <tr>
            <th></th>
            {#if htmlType === "symbol"}
              <th class="border">Symbol</th>
              <th class="border">HTML Representation</th>
            {:else if htmlType === "theoremDescription"}
              <th class="border">Theorem Label</th>
              <th class="border">HTML Snippet</th>
            {:else if htmlType === "headerDescription"}
              <th class="border">Header Position</th>
              <th class="border">HTML Snippet</th>
            {/if}
          </tr>
        </thead>
        <tbody>
          {#each invalidHtml as invalidHtmlRep, i}
            {#if page * 10 <= i && i < (page + 1) * 10}
              <tr>
                <td class="border">{i + 1}</td>
                <td class="border">{invalidHtmlRep[0]}</td>
                <td class="border">{invalidHtmlRep[1]}</td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      <div class="flex flex-row justify-center mt-2">
        <div class="px-2">
          <RoundButton onclick={invalidHtmlPreviousPage} disabled={page === 0}>Previous Page</RoundButton>
        </div>
        <div class="px-2">
          <RoundButton onclick={invalidHtmlNextPage} disabled={page >= Math.floor((invalidHtml.length - 1) / 10)}>Next Page</RoundButton>
        </div>
      </div>
    </div>
  </div>
{/if}
