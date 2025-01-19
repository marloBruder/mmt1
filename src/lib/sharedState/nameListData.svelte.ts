import { invoke } from "@tauri-apps/api/core";
import type { NameListHeader } from "./model.svelte";

class NameListData {
  #inProgressTheoremNames: string[] = $state([]);

  async load() {
    await this.loadInProgressTheoremNames();
  }

  async loadInProgressTheoremNames() {
    this.#inProgressTheoremNames = await invoke("get_in_progress_theorem_names_local");
  }

  addInProgressTheoremName(name: string) {
    this.#inProgressTheoremNames.push(name);
  }

  changeInProgressTheoremName(oldName: string, newName: string) {
    for (let [i, theoremName] of this.#inProgressTheoremNames.entries()) {
      if (theoremName == oldName) {
        this.#inProgressTheoremNames[i] = newName;
      }
    }
  }

  removeInProgressTheoremName(name: string) {
    for (let [i, theoremName] of this.#inProgressTheoremNames.entries()) {
      if (theoremName == name) {
        this.#inProgressTheoremNames.splice(i, 1);
      }
    }
  }

  // // Returns true if the name has not been used before and is a valid metamath label, as in:
  // // "any combination of letters, digits, and the characters hyphen, underscore, and period"
  // validNewName(name: string) {
  //   return /^[a-zA-Z0-9\-_\.]+$/.test(name) && !this.nameExists(name);
  // }

  // nameExists(name: string) {
  //   return this.#theoremNames.find((otherName) => name === otherName) != undefined || this.#inProgressTheoremNames.find((otherName) => name === otherName) != undefined;
  // }

  resetLists() {
    this.#inProgressTheoremNames = [];
  }

  get inProgressTheoremNames() {
    return this.#inProgressTheoremNames;
  }
}

let nameListData = new NameListData();

export { nameListData };
