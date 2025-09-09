import type { HeaderPath } from "./model.svelte";

class Util {
  headerPathToStringRep(headerPath: HeaderPath): string {
    let stringRep = "";
    for (let pos of headerPath.path) {
      stringRep = stringRep + (pos + 1) + ".";
    }
    stringRep = stringRep.slice(0, stringRep.length - 1);
    return stringRep;
  }

  clone(object: any): any {
    return JSON.parse(JSON.stringify(object));
  }
}

let util = new Util();

export { util };
