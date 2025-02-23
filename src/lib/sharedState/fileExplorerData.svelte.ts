import { invoke } from "@tauri-apps/api/core";
import type { Folder, FolderRepresentation } from "./model.svelte";

class FileExplorerData {
  #folder: Folder = $state({ name: "No folder opened", content: null });

  async openFolder(folder: Folder, folderPath: string) {
    let folderRep = (await invoke("get_subfolder", { relativePath: folderPath })) as FolderRepresentation;
    folder.content = {
      fileNames: folderRep.fileNames,
      subfolders: folderRep.subfolderNames.map((subfolderName) => {
        return {
          name: subfolderName,
          content: null,
        };
      }),
    };
  }

  closeFolder(folder: Folder) {
    folder.content = null;
  }

  resetData() {
    this.#folder = { name: "No folder opened", content: null };
  }

  resetDataWithFirstFolder(topFolderName: string, folder: FolderRepresentation) {
    this.#folder = {
      name: topFolderName,
      content: {
        fileNames: folder.fileNames,
        subfolders: folder.subfolderNames.map((subfolderName) => {
          return {
            name: subfolderName,
            content: null,
          };
        }),
      },
    };
  }

  get folder() {
    return this.#folder;
  }
}

let fileExplorerData = new FileExplorerData();

export { fileExplorerData };
