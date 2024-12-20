import { invoke } from "@tauri-apps/api/core";
import type { Theorem } from "../model.svelte";
import { inProgressTheoremData } from "./inProgressTheoremData.svelte";

class TheoremData {
  #theorems: Theorem[] = $state([]);

  convertToTheorem(localID: number) {
    let inProgressTheorem = inProgressTheoremData.getTheoremByID(localID);
    if (inProgressTheorem) {
      invoke("text_to_axium", { text: inProgressTheorem.text }).then((theoremUnknown) => {
        this.#theorems.push(theoremUnknown as Theorem);
        inProgressTheoremData.deleteTheorem(localID);
      });
    }
  }

  addTheoremLocal(theorem: Theorem) {
    this.#theorems.push(theorem);
  }

  getTheoremByName(name: string): Theorem | null {
    for (let theorem of this.#theorems) {
      if (theorem.name == name) {
        return theorem;
      }
    }
    return null;
  }

  get theorems() {
    return this.#theorems;
  }
}

let theoremData = new TheoremData();

export { theoremData };
