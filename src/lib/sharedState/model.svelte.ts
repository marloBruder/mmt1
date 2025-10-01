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

export interface ColorInformation {
  typecode: string;
  variables: string[];
  color: string;
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

export type DatabaseElementPageData = EmptyPageData | HeaderPageData | CommentPageData | ConstantsPageData | VariablesPageData | FloatingHypothesisPageData | TheoremPageData;

export interface EmptyPageData {
  discriminator: "EmptyPageData";
}

export interface HeaderPageData {
  headerPath: string;
  title: string;
  description: string;
  discriminator: "HeaderPageData";
}

export interface CommentPageData {
  commentPath: string;
  comment: Comment;
  discriminator: "CommentPageData";
}

export interface ConstantsPageData {
  constants: Constant[];
  discriminator: "ConstantsPageData";
}

export interface VariablesPageData {
  variables: [Variable, string][];
  discriminator: "VariablesPageData";
}

export interface FloatingHypothesisPageData {
  floatingHypothesis: FloatingHypothesis;
  discriminator: "FloatingHypothesisPageData";
}

export interface TheoremPageData {
  theorem: Theorem;
  theoremNumber: number;
  proofLines: ProofLine[];
  previewErrors: [boolean, boolean, boolean, boolean][] | null;
  previewDeletedMarkers: [boolean][] | null;
  previewConfirmations: boolean[] | null;
  previewConfirmationsRecursive: boolean[] | null;
  previewUnifyMarkers: [boolean, boolean, boolean, boolean][] | null;
  lastTheoremLabel: string | null;
  nextTheoremLabel: string | null;
  axiomDependencies: [string, number][];
  definitionDependencies: [string, number][];
  references: [string, number][];
  descriptionParsed: ParsedDescriptionSegment[];
  discriminator: "TheoremPageData";
}

export interface ProofLine {
  stepName: string;
  hypotheses: string[];
  reference: string;
  referenceNumber: number | null;
  indention: number;
  assertion: string;
}

export type ParsedDescriptionSegment = DescriptionText | DescriptionMathMode | DescriptionLabel | DescriptionLink | DescriptionItalic | DescriptionSubscript | DescriptionHtml | DescriptionHtmlCharacterRef;

export interface DescriptionText {
  text: string;
  discriminator: "DescriptionText";
}

export interface DescriptionMathMode {
  expression: string;
  discriminator: "DescriptionMathMode";
}

export interface DescriptionLabel {
  label: string;
  theoremNumber: number;
  discriminator: "DescriptionLabel";
}

export interface DescriptionLink {
  url: string;
  discriminator: "DescriptionLink";
}

export interface DescriptionItalic {
  italic: string;
  discriminator: "DescriptionItalic";
}

export interface DescriptionSubscript {
  subscript: string;
  discriminator: "DescriptionSubscript";
}

export interface DescriptionHtml {
  html: string;
  discriminator: "DescriptionHtml";
}

export interface DescriptionHtmlCharacterRef {
  charRef: string;
  discriminator: "DescriptionHtmlCharacterRef";
}

export interface TheoremListData {
  list: ListEntry[];
  pageAmount: number;
  pageLimits: [number, number][] | null;
}

export type ListEntry = HeaderListEntry | CommentListEntry | ConstantListEntry | VariableListEntry | FloatingHypothesisListEntry | TheoremListEntry;

export interface HeaderListEntry {
  headerPath: string;
  title: string;
  discriminator: "HeaderListEntry";
}

export interface CommentListEntry {
  commentPath: string;
  text: string;
  discriminator: "CommentListEntry";
}

export interface ConstantListEntry {
  constants: string;
  discriminator: "ConstantListEntry";
}

export interface VariableListEntry {
  variables: string;
  discriminator: "VariableListEntry";
}

export interface FloatingHypothesisListEntry {
  label: string;
  typecode: string;
  variable: string;
  discriminator: "FloatingHypothesisListEntry";
}

export interface TheoremListEntry {
  label: string;
  theoremNumber: number;
  hypotheses: string[];
  assertion: string;
  descriptionParsed: ParsedDescriptionSegment[];
  discriminator: "TheoremListEntry";
}

export interface Folder {
  name: string;
  content: { fileNames: string[]; subfolders: Folder[] } | null;
}

export interface FolderRepresentation {
  fileNames: string[];
  subfolderNames: string[];
}

export interface DetailedError {
  errorType: string;
  startLineNumber: number;
  startColumn: number;
  endLineNumber: number;
  endColumn: number;
}
