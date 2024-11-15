class EditorTabs {
  #nextID = 1;

  #tabs = $state([{ id: 0, name: "test", text: "TTTTTEEEXXXTT" }]);

  // id of opened tab or NaN, if no editor tab is opened
  openedTabID = $state(NaN);

  addTab = () => {
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

  get tabs() {
    return this.#tabs;
  }
}

export default new EditorTabs();
