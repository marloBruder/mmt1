<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import TitleBarDropdown from "./TitleBarDropdown.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { FolderRepresentation, HeaderRepresentation, HtmlRepresentation } from "$lib/sharedState/model.svelte";
  import { explorerData } from "$lib/sharedState/explorerData.svelte";
  import { htmlData } from "$lib/sharedState/htmlData.svelte";
  import { fileExplorerData } from "$lib/sharedState/fileExplorerData.svelte";

  const appWindow = getCurrentWindow();

  let minimizeClick = () => {
    appWindow.minimize();
  };

  let maximizeClick = () => {
    appWindow.toggleMaximize();
  };

  let closeClick = () => {
    appWindow.close();
  };

  let fileDropdownButtons = [
    {
      title: "Open Folder",
      buttonClick: async () => {
        const folderPath = await open({ multiple: false, directory: true });

        if (folderPath) {
          let folderRep = (await invoke("open_folder", { folderPath })) as FolderRepresentation;
          let folderPathSplit = folderPath.split("\\");
          fileExplorerData.resetDataWithFirstFolder(folderPathSplit[folderPathSplit.length - 1], folderRep);
        }
      },
    },
    {
      title: "Close Folder",
      buttonClick: async () => {
        await invoke("close_folder");
        fileExplorerData.resetData();
      },
    },
    { title: "Exit", buttonClick: closeClick },
  ];

  let metamathDropdownButtons = [
    {
      title: "Open Metamath Database",
      buttonClick: async () => {
        const filePath = await open({ multiple: false, directory: false, filters: [{ name: "Metamath Database", extensions: ["mm"] }] });

        if (filePath) {
          let [topHeaderRep, htmlReps]: [HeaderRepresentation, HtmlRepresentation[]] = await invoke("open_metamath_database", { mmFilePath: filePath });
          explorerData.resetExplorerWithFirstHeader(topHeaderRep);
          htmlData.loadLocal(htmlReps);
        }
      },
    },
  ];
</script>

<div class="h-8 w-screen flex justify-between" data-tauri-drag-region>
  <div class="pl-4 h-full flex items-center">
    <span class="text-xl pr-2">mmt1</span>
    <TitleBarDropdown title="File" buttons={fileDropdownButtons}></TitleBarDropdown>
    <TitleBarDropdown title="Metamath" buttons={metamathDropdownButtons}></TitleBarDropdown>
  </div>
  <div class="flex">
    <button class="mx-4" onclick={minimizeClick}>MIN</button>
    <button class="mx-4" onclick={maximizeClick}>MAX</button>
    <button class="mx-4" onclick={closeClick}>CLOSE</button>
  </div>
</div>
