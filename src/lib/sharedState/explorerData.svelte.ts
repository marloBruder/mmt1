import { invoke } from "@tauri-apps/api/core";
import type { HeaderPath, HeaderRepresentation, NameListHeader, TheoremPath } from "./model.svelte";

class ExplorerData {
  #theoremListHeader: NameListHeader = $state({ title: "Explorer:", opened: false, theoremNames: [], subHeaders: [] });

  // intoHeader is given seperately from headerPath for performance reasons
  // Make sure intoHeader is located at headerPath, else there will be bugs
  // Will only load header if it was previously empty
  async loadHeader(headerPath: HeaderPath, intoHeader: NameListHeader) {
    let dataUnknown = await invoke("get_header_local", { headerPath: { path: headerPath.path } });
    let headerRepresentation = dataUnknown as HeaderRepresentation;
    intoHeader.theoremNames = headerRepresentation.theoremNames;
    intoHeader.subHeaders = headerRepresentation.subHeaderNames.map((title) => {
      return { title, opened: false, theoremNames: [], subHeaders: [] };
    });
  }

  async loadHeaderPath(headerPath: HeaderPath): Promise<[NameListHeader, boolean]> {
    let currentHeader = this.#theoremListHeader;
    let currentHeaderPath: HeaderPath = { path: [] };
    let lastOpened = false;

    if (!currentHeader.opened) {
      await this.loadHeader(currentHeaderPath, currentHeader);
      currentHeader.opened = true;
      lastOpened = true;
    }

    for (let index of headerPath.path) {
      if (0 <= index && index < currentHeader.subHeaders.length) {
        currentHeader = currentHeader.subHeaders[index];
        currentHeaderPath.path.push(index);

        if (!currentHeader.opened) {
          await this.loadHeader(currentHeaderPath, currentHeader);
          currentHeader.opened = true;
          lastOpened = true;
        } else {
          lastOpened = false;
        }
      } else {
        throw Error("Invalid Header Path!");
      }
    }

    return [currentHeader, lastOpened];
  }

  async addTheoremName(theoremPath: TheoremPath, name: string) {
    let [header, lastOpened] = await this.loadHeaderPath(theoremPath.headerPath);
    if (!lastOpened) {
      header.theoremNames.splice(theoremPath.theoremIndex, 0, name);
    }
  }

  unloadHeader(header: NameListHeader) {
    header.subHeaders = [];
    header.theoremNames = [];
  }

  resetExplorer() {
    this.#theoremListHeader = { title: "Explorer:", opened: false, theoremNames: [], subHeaders: [] };
  }

  resetExplorerWithFirstHeader(headerRepresentation: HeaderRepresentation) {
    this.#theoremListHeader = {
      title: "Explorer:",
      opened: true,
      theoremNames: headerRepresentation.theoremNames,
      subHeaders: headerRepresentation.subHeaderNames.map((subHeaderName) => {
        return { title: subHeaderName, opened: false, theoremNames: [], subHeaders: [] };
      }),
    };
  }

  get theoremListHeader() {
    return this.#theoremListHeader;
  }
}

let explorerData = new ExplorerData();

export { explorerData };
