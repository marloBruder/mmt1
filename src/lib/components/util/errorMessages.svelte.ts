export function getErrorMessage(errorType: string): string {
  switch (errorType) {
    case "WhitespaceBeforeFirstTokenError": {
      return "Statements can't have trailing whitespace.\n\n(This error only shows before the first statement, because other lines with trailing whitespace continue the previous statement.)";
    }
    case "TooManyConstStatementsError": {
      return "You can only declare one $c statement per mmp file.\n\n(But you can declare multiple constants in one $c statement.)";
    }
    case "EmptyConstStatementError": {
      return "Empty $c statement.";
    }
    case "EmptyVarStatementError": {
      return "Empty $v statement.";
    }
    case "FloatHypStatementFormatError": {
      return "$f statements must be followed by exactly 3 tokens: The label, the typecode and the variable.";
    }
    case "MultipleMmpLabelsError": {
      return "There can be at most one $theorem, $axiom or $header statement per mmp file.";
    }
    case "MissingTheoremLabelError":
    case "TooManyTheoremLabelTokensError": {
      return "$theorem statements must be followed by exactly one token: The label of the theorem.";
    }
    case "MissingAxiomLabelError":
    case "TooManyAxiomLabelTokensError": {
      return "$axiom statements must only be followed by exactly one token: The label of the axiom.";
    }
    case "TooFewHeaderTokensError": {
      return "$header statements must be followed by the header path and the header title.\n\nExample: $header 3.1.2 Test header";
    }
    case "ZeroOrOneSymbolDisjError": {
      return "$d statements must be followed by at least 2 variables.";
    }
    case "MultipleAllowDiscouragedError": {
      return "There should be at most one $allowdiscouraged statement per mmp file.";
    }
    case "TokensAfterAllowDiscouragedError": {
      return "$allowdiscouraged statements should not be followed by any tokens.";
    }
    case "MultipleLocateAfterError": {
      return "There can only be one $locateafter, $locateafterconst or $locateaftervar statement per mmp file.";
    }
    case "TooFewLocateAfterTokensError":
    case "TooManyLocateAfterTokensError": {
      return "$locateafter statements must be followed by exactly one token: The label of the theorem or floating hypothesis the content of the mmp file should be located after.";
    }
    case "TooFewLocateAfterConstTokensError":
    case "TooManyLocateAfterConstTokensError": {
      return "$locateafterconst statements must be followed by exactly one token: The constant which statement the content of the mmp file should be located after.";
    }
  }

  return "You should not be seeing this error message";
}
