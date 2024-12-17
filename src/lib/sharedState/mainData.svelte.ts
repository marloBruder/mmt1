import { invoke } from "@tauri-apps/api/core";

class InProgressTheoremData {
  #nextID = 1;

  #theorems: { id: number; name: string; text: string }[] = $state([]);

  addDefaultTheorem = () => {
    while (this.nameExists(this.#nextID, "Theorem " + this.#nextID)) {
      this.#nextID++;
    }
    this.#theorems.push({ id: this.#nextID, name: "Theorem " + this.#nextID, text: "" });

    invoke("add_in_progress_theorem", { name: "Theorem " + this.#nextID, text: "" });

    this.#nextID++;
  };

  addTheorem = (name: string, text: string) => {
    this.#theorems.push({ id: this.#nextID, name, text });
    this.#nextID++;
  };

  clearTheorems = () => {
    this.#theorems = [];
    this.#nextID = 1;
  };

  getTheoremByName = (name: string) => {
    for (let tab of this.#theorems) {
      if (tab.name == name) {
        return tab;
      }
    }
    return null;
  };

  getTheoremByID = (id: number) => {
    for (let tab of this.#theorems) {
      if (tab.id == id) {
        return tab;
      }
    }
    return null;
  };

  // Checks whether there exists a tab with different id, but the same name
  nameExists = (id: number, name: string): boolean => {
    for (let t of this.#theorems) {
      if (t.id != id && t.name == name) {
        return true;
      }
    }
    return false;
  };

  get theorems() {
    return this.#theorems;
  }
}

export default new InProgressTheoremData();
