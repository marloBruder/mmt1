class GlobalState {
  databaseBeingOpened: string = $state("");
}

let globalState = new GlobalState();

export { globalState };
