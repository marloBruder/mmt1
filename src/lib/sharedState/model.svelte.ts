export interface MetamathData {
  in_progress_theorems: InProgressTheorem[];
  theorems: Theorem[];
}

export interface InProgressTheorem {
  name: string;
  text: string;
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

export interface Variable {
  type: string;
  name: string;
}
