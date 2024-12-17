import { invoke } from "@tauri-apps/api/core";
import { EditorTabClass, tabManager } from "./tabData.svelte";

class InProgressTheoremData {
  #nextID = 1;

  #theorems: { id: number; name: string; text: string }[] = $state([]);

  addDefaultTheorem() {
    while (this.nameExists(this.#nextID, "Theorem " + this.#nextID)) {
      this.#nextID++;
    }
    this.#theorems.push({ id: this.#nextID, name: "Theorem " + this.#nextID, text: "" });

    invoke("add_in_progress_theorem", { name: "Theorem " + this.#nextID, text: "" });

    this.#nextID++;
  }

  addTheoremLocal(name: string, text: string) {
    this.#theorems.push({ id: this.#nextID, name, text });
    this.#nextID++;
  }

  deleteTheorem(localID: number) {
    for (let [index, theorem] of this.#theorems.entries()) {
      if (theorem.id == localID) {
        invoke("delete_in_progress_theorem", { name: theorem.name });
        tabManager.closeSameTab(new EditorTabClass(localID));
        this.#theorems.splice(index, 1);
        return;
      }
    }
  }

  resetTheoremsLocal() {
    this.#theorems = [];
    this.#nextID = 1;
  }

  getTheoremByID(id: number) {
    for (let tab of this.#theorems) {
      if (tab.id == id) {
        return tab;
      }
    }
    return null;
  }

  validID(id: number): boolean {
    for (let tab of this.#theorems) {
      if (tab.id == id) {
        return true;
      }
    }
    return false;
  }

  // Checks whether there exists a tab with different id, but the same name
  nameExists(id: number, name: string): boolean {
    for (let t of this.#theorems) {
      if (t.id != id && t.name == name) {
        return true;
      }
    }
    return false;
  }

  get theorems() {
    return this.#theorems;
  }
}

let inProgressTheoremData = new InProgressTheoremData();

export { inProgressTheoremData };
