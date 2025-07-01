import { invoke } from "@tauri-apps/api/core";
import type { ColorInformation, HtmlRepresentation } from "./model.svelte";

class HtmlData {
  // Maps each math symbol to a tuple where:
  //  The first element is the html representation
  //  The second element is:
  //   0 if it is not a varialbe or if it is a variable which typecode does not have a color specified
  //   i, where the symbols typecode is the i-th typecode to have it's color specified
  #htmlRepresentations: Map<string, [string, number]> = $state(new Map());

  // async load() {
  //   let htmlRepresentations = (await invoke("get_html_representations_local")) as HtmlRepresentation[];
  //   this.loadLocal(htmlRepresentations, []);
  // }

  loadLocal(htmlRepresentations: HtmlRepresentation[], colorInformation: ColorInformation[]) {
    this.#htmlRepresentations.clear();
    for (let htmlRepresentation of htmlRepresentations) {
      let typecode = 0;
      for (let [i, information] of colorInformation.entries()) {
        if (information.variables.some((value) => value === htmlRepresentation.symbol)) {
          typecode = i + 1;
          break;
        }
      }

      this.#htmlRepresentations.set(htmlRepresentation.symbol, [htmlRepresentation.html, typecode]);
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
