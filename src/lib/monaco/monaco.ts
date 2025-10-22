import type { ColorInformation } from "$lib/sharedState/model.svelte";
import * as monaco from "monaco-editor";

// Import the workers in a production-safe way.
// This is different than in Monaco's documentation for Vite,
// but avoids a weird error ("Unexpected usage") at runtime
import editorWorker from "monaco-editor/esm/vs/editor/editor.worker?worker";
import cssWorker from "monaco-editor/esm/vs/language/css/css.worker?worker";
import htmlWorker from "monaco-editor/esm/vs/language/html/html.worker?worker";
import jsonWorker from "monaco-editor/esm/vs/language/json/json.worker?worker";
import tsWorker from "monaco-editor/esm/vs/language/typescript/ts.worker?worker";

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

// let test: ColorInformation[] = [
//   { typecode: "wff", variables: ["ph", "ps", "ch", "th", "ta", "et", "ze", "si", "rh", "mu", "la", "ka"], color: "337DFF" },
//   { typecode: "setvar", variables: ["x", "y", "z", "w", "v", "u", "t"], color: "ff0000" },
//   { typecode: "class", variables: ["A", "B", "C", "D", "P", "Q", "R", "S"], color: "cc33cc" },
// ];

let colorInformationToKeywords = (colorInformation: ColorInformation[]): any => {
  let res: any = {};

  for (let information of colorInformation) {
    res["$" + information.typecode] = information.variables;
  }

  return res;
};

let colorInformationToCases = (colorInformation: ColorInformation[]): any => {
  let res: any = {};

  for (let information of colorInformation) {
    res["@$" + information.typecode] = "$" + information.typecode;
    // res["@default"] = "token";
  }

  return res;
};

let colorInformationToRules = (colorInformation: ColorInformation[]): { token: string; foreground: string }[] => {
  let res: { token: string; foreground: string }[] = [];

  for (let information of colorInformation) {
    res.push({ token: "$" + information.typecode, foreground: information.color });
  }

  return res;
};

monaco.languages.register({ id: "mmp" });

export let setEditorSyntaxHighlighting = (colorInformation: ColorInformation[]) => {
  monaco.languages.setMonarchTokensProvider("mmp", {
    ...colorInformationToKeywords(colorInformation),
    keywords: ["$theorem", "$axiom", "$c", "$v", "$f", "$header", "$locateafter", "$locateaftervar", "$locateafterconst", "$locateafterheader", "$locateaftercomment", "$locateafterstart", "$allowdiscouraged", "$allowincomplete", "$d", "$="],
    keywordsWithoutVarColor: ["$theorem", "$axiom", "$header", "$locateafter", "$locateafterheader", "$locateaftercomment", "$locateafterstart", "$allowdiscouraged", "$allowincomplete", "$="],
    tokenizer: {
      root: [{ include: "line" }],

      comment: [{ include: "line" }, [/.*/, "comment"]],

      line: [
        [/^\*\S*/, "comment", "@comment"],
        [/^\$[\w$]+/, { cases: { "@keywordsWithoutVarColor": { token: "keyword", next: "@lineWithoutVarColor" }, "@keywords": { token: "keyword", next: "@line" }, "@default": { token: "error", next: "@root" } } }],
        [/^\S+/, "linePrefix", "@line"],
        [/\S+/, { cases: colorInformationToCases(colorInformation) }],
      ],

      lineWithoutVarColor: [
        [/^\*\S*/, "comment", "@comment"],
        [/^\$[\w$]+/, { cases: { "@keywordsWithoutVarColor": { token: "keyword", next: "@lineWithoutVarColor" }, "@keywords": { token: "keyword", next: "@line" }, "@default": { token: "error", next: "@root" } } }],
        [/^\S+/, "linePrefix", "@line"],
      ],
    },
  });

  monaco.editor.defineTheme("mmp-theme", {
    colors: {
      "editor.background": "#262335",
      "editor.foreground": "#FFFFFF",
      // "diffEditor.insertedTextBackground": "#39634C",
      "diffEditor.insertedTextBackground": "#005030",
    },
    base: "vs-dark",
    inherit: false,
    rules: [
      { token: "comment", foreground: "777777" },
      { token: "linePrefix", foreground: "FFFFFF" },
      { token: "keyword", foreground: "d4922f" }, //"fc8005" },
      { token: "error", foreground: "fc0515" },
      ...colorInformationToRules(colorInformation),
    ],
  });
};

setEditorSyntaxHighlighting([]);

export default monaco;
