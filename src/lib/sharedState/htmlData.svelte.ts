import { invoke } from "@tauri-apps/api/core";
import type { ColorInformation, HtmlRepresentation } from "./model.svelte";

class HtmlData {
  // Maps each math symbol to a tuple where:
  //  The first element is the html representation
  //  The second element is:
  //   0 if it is not a varialbe or if it is a variable which typecode does not have a color specified
  //   i, where the symbols typecode is the i-th typecode to have it's color specified
  #htmlRepresentations: Map<string, [string, number]> = $state(new Map());

  async load() {
    let [htmlRepresentations, colorInfo] = (await invoke("get_html_representations_local")) as [HtmlRepresentation[], ColorInformation[]];
    this.loadLocal(htmlRepresentations, colorInfo);
  }

  loadLocal(htmlRepresentations: HtmlRepresentation[], colorInformation: ColorInformation[]) {
    addCustomStylesheetForVariableColorOverriding(colorInformation);

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

export function addCustomStylesheetForVariableColorOverriding(colorInformation: ColorInformation[]) {
  // Add custom stylesheet to override color of variable html representations
  let existing_stylesheet = document.getElementById("custom-syntax-highlighting-stylesheet");
  if (existing_stylesheet) {
    document.head.removeChild(existing_stylesheet);
  }

  let style = "";

  for (let [i, information] of colorInformation.entries()) {
    style =
      style +
      `.custom-variable-color-${i + 1} * {
    color: #${information.color} !important;    
  }
    
  `;
  }

  let stylesheet = document.createElement("style");
  stylesheet.id = "custom-syntax-highlighting-stylesheet";
  stylesheet.textContent = style;
  document.head.appendChild(stylesheet);
}

let htmlData = new HtmlData();

export { htmlData };
