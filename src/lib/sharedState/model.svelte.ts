export interface MetamathData {
  constants: Constant[];
  variables: Variable[];
  theorems: Theorem[];
  in_progress_theorems: InProgressTheorem[];
}

export interface Constant {
  symbol: string;
}

export interface Variable {
  symbol: string;
}

export interface Theorem {
  name: string;
  description: string;
  disjoints: string[];
  hypotheses: Hypothesis[];
  assertion: string;
  proof: string | null;
}

export interface Hypothesis {
  label: string;
  hypothesis: string;
}

export interface InProgressTheorem {
  name: string;
  text: string;
}
