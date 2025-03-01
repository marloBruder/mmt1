import { invoke } from "@tauri-apps/api/core";
import type { HeaderPath, HeaderRepresentation, NameListHeader, TheoremPath } from "./model.svelte";

class ExplorerData {
  #theoremListHeader: NameListHeader = $state({ title: "Explorer:", content: null });

  // intoHeader is given seperately from headerPath for performance reasons
  // Make sure intoHeader is located at headerPath, else there will be bugs
  async loadHeader(headerPath: HeaderPath, intoHeader: NameListHeader) {
    let headerRepresentation = (await invoke("get_header_local", { headerPath: { path: headerPath.path } })) as HeaderRepresentation;
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

    if (currentHeader.content == null) {
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

  async addTheoremName(theoremPath: TheoremPath, title: string) {
    let [header, lastOpened] = await this.loadHeaderPath(theoremPath.headerPath);
    if (!lastOpened) {
      header.content!.contentTitles.splice(theoremPath.theoremIndex, 0, { contentType: "TheoremStatement", title });
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
