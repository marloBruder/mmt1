import * as monaco from "monaco-editor";

// Import the workers in a production-safe way.
// This is different than in Monaco's documentation for Vite,
// but avoids a weird error ("Unexpected usage") at runtime
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";
import { comment, root } from "postcss";

self.MonacoEnvironment = {
  getWorker: function (_: string, label: string) {
    switch (label) {
      case "json":
        return new jsonWorker();
      case "css":
      case "scss":
      case "less":
        return new cssWorker();
      case "html":
      case "handlebars":
      case "razor":
        return new htmlWorker();
      case "typescript":
      case "javascript":
        return new tsWorker();
      default:
        return new editorWorker();
    }
  },
};

interface ColorInformation {
  information: { typecode: string; variables: string[]; color: string }[];
}

let test: ColorInformation = {
  information: [
    { typecode: "wff", variables: ["ph", "ps", "ch", "th", "ta", "et", "ze", "si", "rh", "mu", "la", "ka"], color: "0000FF" },
    { typecode: "setvar", variables: ["x", "y", "z", "w", "v", "u", "t"], color: "ff0000" },
    { typecode: "class", variables: ["A", "B", "C", "D", "P", "Q", "R", "S"], color: "cc33cc" },
  ],
};

let colorInformationToKeywords = (colorInformation: ColorInformation): any => {
  let res: any = {};

  for (let information of colorInformation.information) {
    res["$" + information.typecode] = information.variables;
  }

  return res;
};

let colorInformationToCases = (colorInformation: ColorInformation): any => {
  let res: any = {};

  for (let information of colorInformation.information) {
    res["@$" + information.typecode] = "$" + information.typecode;
  }

  return res;
};

let colorInformationToRules = (colorInformation: ColorInformation): { token: string; foreground: string }[] => {
  let res: { token: string; foreground: string }[] = [];

  for (let information of colorInformation.information) {
    res.push({ token: "$" + information.typecode, foreground: information.color });
  }

  return res;
};

monaco.languages.register({ id: "mmp" });

export let setSyntaxHighlighting = (colorInformation: ColorInformation) => {
  monaco.languages.setMonarchTokensProvider("mmp", {
    ...colorInformationToKeywords(colorInformation),
    keywords: ["$theorem", "$axiom", "$c", "$v", "$f", "$header", "$comment", "$locateafter", "$locateaftervar", "$locateafterconst", "$allowdiscouraged", "$d"],
    tokenizer: {
      root: [{ include: "@whitespace" }, { include: "line" }],

      comment: [{ include: "line" }, [/.*/, "comment"]],

      whitespace: [
        [/[ \t\r\n]+/, "white"],
        [/^\*/, "comment", "@comment"],
      ],

      line: [
        [/^\S*:\S*:\S*/, "linePrefix", "@root"],
        [/^\$[\w$]+/, { cases: { "@keywords": { token: "keyword", next: "@root" }, "@default": { token: "error", next: "@root" } } }],
        [/\S+/, { cases: colorInformationToCases(colorInformation) }],
        [/^\S+/, "error", "@root"],
      ],
      // mmj2StepPop: [[/^\S*:\S*:\S*/, "test", "@pop"]],
    },
  });

  monaco.editor.defineTheme("mmp-theme", {
    colors: {},
    base: "vs",
    inherit: false,
    rules: [
      { token: "comment", foreground: "777777" },
      { token: "linePrefix", foreground: "000000" },
      { token: "keyword", foreground: "000000" }, //"fc8005" },
      { token: "error", foreground: "fc0515" },
      ...colorInformationToRules(colorInformation),
    ],
  });
};

// setSyntaxHighlighting({ information: [] });
setSyntaxHighlighting(test);

export default monaco;
