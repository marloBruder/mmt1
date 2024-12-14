class EditorTabs {
  #nextID = 1;

  #tabs: { id: number; name: string; text: string }[] = $state([]);

  // id of opened tab or NaN, if no editor tab is opened
  openedTabID = $state(NaN);

  addTab = () => {
    while (this.nameExists(this.#nextID, "Theorem " + this.#nextID)) {
      this.#nextID++;
    }
    this.#tabs.push({ id: this.#nextID, name: "Theorem " + this.#nextID, text: "" });
    this.#nextID++;
  };

  getTabByID = (id: number) => {
    for (let tab of this.#tabs) {
      if (tab.id == id) {
        return tab;
      }
    }
    return null;
  };

  // Checks whether there exists a tab with different id, but the same name
  nameExists = (id: number, name: string): boolean => {
    for (let t of this.#tabs) {
      if (t.id != id && t.name == name) {
        return true;
      }
    }
    return false;
  };

  get tabs() {
    return this.#tabs;
  }
}

export default new EditorTabs();
