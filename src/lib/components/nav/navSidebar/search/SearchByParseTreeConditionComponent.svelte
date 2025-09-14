<script lang="ts">
  import SelectDropdown from "$lib/components/util/SelectDropdown.svelte";
  import CloseIcon from "$lib/icons/titleBar/CloseIcon.svelte";
  import type { SearchByParseTreeCondition } from "$lib/sharedState/searchData.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { searchByParseTreeCondition, onRemoveClick, validInput = $bindable() }: { searchByParseTreeCondition: SearchByParseTreeCondition; onRemoveClick: () => void; validInput: boolean } = $props();

  let searchByParseTreeSearchTargetOptions = [
    { label: "any hypothesis,", value: "anyHypothesis" },
    { label: "all hypotheses,", value: "allHypotheses" },
    { label: "the assertion,", value: "assertion" },
    { label: "any hypothesis or the assertion,", value: "anyExpressions" },
    { label: "all hypotheses and the assertion,", value: "allExpressions" },
  ];

  let searchByParseTreeSearchConditionOptions = [
    { label: "matching", value: "matches" },
    { label: "containing", value: "contains" },
  ];

  let oninput = async () => {
    validInput = await invoke("search_by_parse_tree_syntax_check", { search: searchByParseTreeCondition.search });
  };
</script>

<div class="pb-2 border rounded-lg mb-2">
  <div class="border-b flex flex-row-reverse">
    <button onclick={onRemoveClick}><CloseIcon></CloseIcon></button>
  </div>
  <div class="p-2">
    In
    <SelectDropdown bind:value={searchByParseTreeCondition.searchTarget} options={searchByParseTreeSearchTargetOptions}></SelectDropdown>
    search for expressions
    <SelectDropdown bind:value={searchByParseTreeCondition.searchCondition} options={searchByParseTreeSearchConditionOptions}></SelectDropdown>
  </div>
  <div class="px-2">
    <input class={"w-full custom-bg-input-color border rounded " + (validInput ? "" : " border-red-500 ")} bind:value={searchByParseTreeCondition.search} {oninput} autocomplete="off" spellcheck="false" />
  </div>
</div>
