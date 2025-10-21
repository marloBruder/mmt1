import { invoke } from "@tauri-apps/api/core";
import type { HeaderContentRepresentation, HeaderPath, HeaderRepresentation, NameListHeader } from "./model.svelte";

class ExplorerData {
  #theoremListHeader: NameListHeader = $state({ title: "Explorer:", content: null });

  // intoHeader is given seperately from headerPath for performance reasons
  // Make sure intoHeader is located at headerPath, else there will be bugs
  async loadHeader(headerPath: HeaderPath, intoHeader: NameListHeader) {
    let headerRepresentation = (await invoke("get_header_local", { headerPath })) as HeaderRepresentation;
    intoHeader.content = {
      contentTitles: headerRepresentation.contentTitles,
      subheaders: headerRepresentation.subheaderTitles.map((title) => {
        return { title, content: null };
      }),
    };
  }

  async loadHeaderPath(headerPath: HeaderPath): Promise<[NameListHeader, boolean]> {
    let currentHeader = this.#theoremListHeader;
    let currentHeaderPath: HeaderPath = { path: [] };
    let lastOpened = false;

    if (currentHeader.content === null) {
      await this.loadHeader(currentHeaderPath, currentHeader);
      lastOpened = true;
    }

    for (let index of headerPath.path) {
      if (0 <= index && index < currentHeader.content!.subheaders.length) {
        currentHeader = currentHeader.content!.subheaders[index];
        currentHeaderPath.path.push(index);

        if (currentHeader.content == null) {
          await this.loadHeader(currentHeaderPath, currentHeader);
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

  addHeader(headerPath: HeaderPath, title: string) {
    let targetHeader = this.#theoremListHeader;

    for (let [i, subheaderI] of headerPath.path.entries()) {
      if (i !== headerPath.path.length - 1) {
        if (targetHeader.content !== null && targetHeader.content.subheaders.length > subheaderI) {
          targetHeader = targetHeader.content.subheaders[subheaderI];
        } else {
          return;
        }
      } else {
        if (targetHeader.content !== null) {
          targetHeader.content.subheaders.splice(subheaderI, 0, { title, content: null });
        }
      }
    }
  }

  addHeaderContent(headerPath: HeaderPath, headerContentI: number, headerContent: HeaderContentRepresentation) {
    let targetHeader = this.#theoremListHeader;

    for (let subheaderI of headerPath.path) {
      if (targetHeader.content !== null && targetHeader.content.subheaders.length > subheaderI) {
        targetHeader = targetHeader.content.subheaders[subheaderI];
      } else {
        return;
      }
    }

    if (targetHeader.content !== null) {
      targetHeader.content.contentTitles.splice(headerContentI, 0, headerContent);
    }
  }

  unloadHeader(header: NameListHeader) {
    header.content = null;
  }

  resetExplorer() {
    this.#theoremListHeader = { title: "Explorer:", content: null };
  }

  resetExplorerWithFirstHeader(headerRepresentation: HeaderRepresentation) {
    this.#theoremListHeader = {
      title: "Explorer:",
      content: {
        contentTitles: headerRepresentation.contentTitles,
        subheaders: headerRepresentation.subheaderTitles.map((title) => {
          return { title, content: null };
        }),
      },
    };
  }

  get theoremListHeader() {
    return this.#theoremListHeader;
  }
}

let explorerData = new ExplorerData();

export { explorerData };
