import monaco from "$lib/monaco/monaco";

export function getErrorMessage(errorType: string): string {
  switch (errorType) {
    case "NonAsciiSymbolError": {
      return ".mmp files may only contain ASCII tokens.";
    }
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
    case "InvalidLabelError": {
      return "Not a valid label. A label token consists of any combination of letters, digits and the characters hyphen, underscore and period.";
    }
    case "MultipleMmpLabelsError": {
      return "There can be at most one $theorem, $axiom or $header statement per mmp file.";
    }
    case "TooFewHeaderTokensError": {
      return "$header statements must be followed by the header path and the header title.\n\nExample: $header 3.1.2 Test header";
    }
    case "MissingCommentPathError":
    case "TooManyCommentPathTokensError": {
      return "$comment statements must only be followed by exactly one token: The path of the comment.\n\nExample: $comment 3.4.2#5 (The fifth comment under header 3.4.2)";
    }
    case "InvalidCommentPathFormatError": {
      return "$comment statements must be followed by the path of the comment.\n\nExample: $comment 3.4.2#5 (The fifth comment under header 3.4.2)";
    }
    case "MissingAxiomLabelError":
    case "TooManyAxiomLabelTokensError": {
      return "$axiom statements must only be followed by exactly one token: The label of the axiom.";
    }
    case "MissingTheoremLabelError":
    case "TooManyTheoremLabelTokensError": {
      return "$theorem statements must be followed by exactly one token: The label of the theorem.";
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
    case "InvalidDollarTokenError": {
      return "Invalid keyword. Step names may not start with '$'.";
    }
    case "InvalidMmpStepPrefixFormatError": {
      return "Each step prefix must be of the format [h]name:hyps:ref, where the h at the beginning indicates that the step is a hypothesis, name is the name of the step, hyps is a comma seperated list of hypotheses names and ref is either the name of the theorem being applied if the step is not a hypothesis or the name of the hypothesis otherwise.";
    }
    case "InvalidMmpStepNameError": {
      return "Step names cannot be empty and must be alphanumeric.";
    }
    case "InvalidMmpStepNameStartsWithHError": {
      return "Step names cannot start with 'h'.";
    }
    case "DuplicateStepNameError": {
      return "Duplicate step name";
    }
    case "HypNameDoesntExistError": {
      return "This is not the name of a previous step.";
    }
    case "DuplicateHypLabelsError": {
      return "Duplicate hypothesis label.";
    }
    case "MissingMmpStepExpressionError": {
      return "Missing expression.";
    }
    case "NonSymbolInExpressionError": {
      return "Not a valid symbol.";
    }
    case "ExpressionParseError": {
      return "Expression could not be successfully parsed. Are you perhaps missing a parenthesis?";
    }
  }

  return "You should not be seeing this error message. Please post a Github issue with your editor content.";
}

export function getErrorSeverity(errorType: string): monaco.MarkerSeverity {
  switch (errorType) {
    case "MissingMmpStepExpressionError":
    case "NonSymbolInExpressionError":
    case "ExpressionParseError":
      return monaco.MarkerSeverity.Warning;
    default:
      return monaco.MarkerSeverity.Error;
  }
}
