export interface MetamathData {
  constants: Constant[];
  variables: Variable[];
  theorems: Theorem[];
  in_progress_theorems: InProgressTheorem[];
}

export interface Comment {
  text: string;
}

export interface Constant {
  symbol: string;
}

export interface Variable {
  symbol: string;
}

export interface FloatingHypothesis {
  label: string;
  typecode: string;
  variable: string;
}

export interface Theorem {
  label: string;
  description: string;
  distincts: string[];
  hypotheses: Hypothesis[];
  assertion: string;
  proof: string | null;
}

export interface Hypothesis {
  label: string;
  expression: string;
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
  contentType: "CommentStatement" | "ConstantStatement" | "VariableStatement" | "FloatingHypothesisStatement" | "TheoremStatement";
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
  lastTheoremLabel: string | null;
  nextTheoremLabel: string | null;
}

export interface ProofLine {
  hypotheses: number[];
  reference: string;
  indention: number;
  assertion: string;
}

export interface TheoremListData {
  list: TheoremListEntry[];
  pageAmount: number;
}

export interface TheoremListEntry {
  label: string;
  theoremNumber: number;
  hypotheses: string[];
  assertion: string;
  description: string;
}

export interface SearchParameters {
  page: number;
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
