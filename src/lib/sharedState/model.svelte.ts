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

export interface FloatingHypotheses {
  label: string;
  typecode: string;
  variable: string;
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

// Header, but instead of having a theorem list, it has a theorem name list
// Aditionally has a opened to store whether the headers ui is opened
// Used to store the explorer state
export interface NameListHeader {
  title: string;
  opened: boolean;
  theoremNames: string[];
  subHeaders: NameListHeader[];
}

// Header, but both theorem and subheader lists have been replaced by names
// Used to recieve data about a singular header from backend
export interface HeaderRepresentation {
  title: string;
  theoremNames: string[];
  subHeaderNames: string[];
}

export interface HeaderPath {
  path: number[];
}

export interface TheoremPath {
  headerPath: HeaderPath;
  theoremIndex: number;
}

export interface HtmlRepresentation {
  symbol: string;
  html: string;
}

export interface TheoremPageData {
  theorem: Theorem;
  proofLines: ProofLine[];
}

export interface ProofLine {
  hypotheses: number[];
  reference: string;
  indention: number;
  assertion: string;
}
