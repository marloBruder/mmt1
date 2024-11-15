class EditorTabs {
  nextID = 1;

  #tabs = $state([{ id: 0, name: "test", text: "TTTTTEEEXXXTT" }]);

  addTab = () => {
    this.#tabs.push({ id: this.nextID, name: "Theorem " + this.nextID, text: "" });
    this.nextID++;
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
