<script lang="ts">
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";
  import FolderHeader from "./editor/FolderHeader.svelte";
</script>

<div class="pt-2">
  <FolderHeader folder={fileExplorerData.folder} folderPath=""></FolderHeader>
</div>
