import { invoke } from "@tauri-apps/api/core";
import type { HtmlRepresentation } from "./model.svelte";

class HtmlData {
  #htmlRepresentations: Map<string, string> = $state(new Map());

  async load() {
    let htmlRepresentations = (await invoke("get_html_representations_local")) as HtmlRepresentation[];
    this.loadLocal(htmlRepresentations);
  }

  loadLocal(htmlRepresentations: HtmlRepresentation[]) {
    this.#htmlRepresentations.clear();
    for (let htmlRepresentation of htmlRepresentations) {
      this.#htmlRepresentations.set(htmlRepresentation.symbol, htmlRepresentation.html);
    }
  }

  getHtml(symbol: string) {
    return this.#htmlRepresentations.get(symbol);
  }

  resetHtmlData() {
    this.#htmlRepresentations.clear();
  }

  get htmlRepresentations() {
    return this.#htmlRepresentations;
  }
}

let htmlData = new HtmlData();

export { htmlData };
