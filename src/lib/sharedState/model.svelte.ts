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
  content: {
    contentTitles: HeaderContentRepresentation[];
    subheaders: NameListHeader[];
  } | null;
}

// Header, but both theorem and subheader lists have been replaced by names
// Used to recieve data about a singular header from backend
export interface HeaderRepresentation {
  title: string;
  contentTitles: HeaderContentRepresentation[];
  subheaderTitles: string[];
}

export interface HeaderContentRepresentation {
  contentType: "ConstantStatement" | "VariableStatement" | "FloatingHypohesisStatement" | "TheoremStatement";
  title: string;
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
  theoremNumber: number;
  proofLines: ProofLine[];
}

export interface ProofLine {
  hypotheses: number[];
  reference: string;
  indention: number;
  assertion: string;
}

export interface TheoremListEntry {
  name: string;
  theoremNumber: number;
  hypotheses: string[];
  assertion: string;
  description: string;
}

export interface SearchParameters {
  start: number;
  amount: number;
  label: string;
}

export interface Folder {
  name: string;
  content: { fileNames: string[]; subfolders: Folder[] } | null;
}

export interface FolderRepresentation {
  fileNames: string[];
  subfolderNames: string[];
}
