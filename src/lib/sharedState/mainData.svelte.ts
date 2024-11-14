let nextID = 1;

export const editorTabs = $state({
  tabs: [{ id: 0, name: "test", text: "TTTTTEEEXXXTT" }],
  addTab: function () {
    this.tabs.push({ id: nextID, name: "Theorem " + nextID, text: "" });
    nextID++;
  },
});
