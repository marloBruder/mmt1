import type { Variable } from "../model.svelte";

class VariableData {
  #variables: Variable[] = $state([]);

  addVariable(variable: Variable) {
    this.addVariableLocal(variable);
  }

  addVariableLocal(variable: Variable) {
    this.#variables.push(variable);
  }

  resetVariablesLocal() {
    this.#variables = [];
  }

  get variables() {
    return this.#variables;
  }
}

let variableData = new VariableData();

export { variableData };
