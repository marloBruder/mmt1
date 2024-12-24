import { invoke } from "@tauri-apps/api/core";

class NameListData {
  #theoremNames: string[] = $state([]);
  #inProgressTheoremNames: string[] = $state([]);

  async load() {
    await this.loadInProgressTheoremNames();
    await this.loadTheoremNames();
  }

  async loadTheoremNames() {
    this.#theoremNames = await invoke("get_theorem_names_local");
  }

  async loadInProgressTheoremNames() {
    this.#inProgressTheoremNames = await invoke("get_in_progress_theorem_names_local");
  }

  addTheoremName(name: string) {
    this.#theoremNames.push(name);
  }

  addInProgressTheoremName(name: string) {
    this.#inProgressTheoremNames.push(name);
  }

  // removeTheoremName(name: string) {
  //   for (let [i, theoremName] of this.#theoremNames.entries()) {
  //     if (theoremName == name) {
  //       this.#theoremNames.splice(i, 1);
  //     }
  //   }
  // }

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

  // Returns true if the name has not been used before and is a valid metamath label, as in:
  // "any combination of letters, digits, and the characters hyphen, underscore, and period"
  validNewName(name: string) {
    return /^[a-zA-Z0-9\-_\.]+$/.test(name) && !this.nameExists(name);
  }

  nameExists(name: string) {
    return this.#theoremNames.find((otherName) => name === otherName) != undefined || this.#inProgressTheoremNames.find((otherName) => name === otherName) != undefined;
  }

  resetLists() {
    this.#inProgressTheoremNames = [];
    this.#theoremNames = [];
  }

  get theoremNames() {
    return this.#theoremNames;
  }

  get inProgressTheoremNames() {
    return this.#inProgressTheoremNames;
  }
}

let nameListData = new NameListData();

export { nameListData };
