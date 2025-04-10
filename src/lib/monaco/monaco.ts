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

monaco.languages.register({ id: "mmp" });

export let setSyntaxHighlighting = () => {
  monaco.languages.setMonarchTokensProvider("mmp", {
    keywords: ["$theorem", "$axiom", "$c", "$v", "$f", "$header", "$locateafter", "$locateaftervar", "$locateafterconst", "$allowdiscouraged", "$d"],
    tokenizer: {
      root: [{ include: "@whitespace" }, { include: "line" }],

      comment: [{ include: "line" }, [/.*/, "comment"]],

      whitespace: [
        [/[ \t\r\n]+/, "white"],
        [/^\*/, "comment", "@comment"],
      ],

      line: [
        [/^\S*:\S*:\S*/, "test", "@root"],
        [/^\$[\w$]+/, { cases: { "@keywords": { token: "keyword", next: "@root" }, "@default": { token: "error", next: "@root" } } }],
        // [/^\$theorem\s|^\$locateafter\s|^\$locateaftervar\s|^\$locateafterconst|^\$allowdiscouraged|^\$d/, "keyword", "@root"],
        [/\|\-/, "keyword", "@root"],
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
      { token: "test", foreground: "6102f9" },
      { token: "keyword", foreground: "fc8005" },
      { token: "error", foreground: "fc0515" },
    ],
  });
};

setSyntaxHighlighting();

export default monaco;
