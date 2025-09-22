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
    // case "InvalidCommentPathFormatError": {
    //   return "$comment statements must be followed by the path of the comment.\n\nExample: $comment 3.4.2#5 (The fifth comment under header 3.4.2)";
    // }
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
      return "Invalid keyword. Step names may not start with or constain with '$'.";
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
      return "Not the name of a previous step.";
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
    case "ConstStatementOutOfPlaceError": {
      return "$c statement should not be here. $c statements are only allowed when describing a constant statement and can not appear alongside $header, $comment, $axiom, $theorem, $v or $f statements.";
    }
    case "VarStatementOutOfPlaceError": {
      return "$v statement should not be here. $v statements are only allowed when describing a variable statement or when adding a temporary variable to an axiom or theorem. $v statements cannot appear alongside $header or $comment statements. Multiple $v statements are only allowed when there is an $axiom or a $theorem statement.";
    }
    case "FloatHypStatementOutOfPlaceError": {
      return "$f statement should not be here. $f statements are only allowed when describing a floating hypothesis or when adding a temporary floating hypothesis to an axiom or theorem. $f statements cannot appear alongside $header or $comment statements. Multiple $f statements are only allowed when there is an $axiom or a $theorem statement.";
    }
    case "AllowDiscouragedOutOfPlaceError": {
      return "$allowdiscouraged statement should not be here. $allowdiscouraged may only appear alongside a $theorem statement.";
    }
    case "LocateAfterOutOfPlaceError": {
      return "Locate after statement should not be here. Locate after statements may not appear alongside $comment or $header statements, as their location in the database is determined by their respective comment or header path.";
    }
    case "DistinctVarOutOfPlaceError": {
      return "$d statement should not be here. $d statements may only appear alongside a $theorem or $axiom statement.";
    }
    case "ProofLinesOutOfPlaceError": {
      return "Proof line should not be here. Proof lines may only appear alongside a $theorem or $axiom statement.";
    }
    case "InvalidHeaderPathFormatError": {
      return "Not a valid header path. Header paths must be a list of dot seperated numbers.\n\nExample: 1.2.3";
    }
    case "InvalidCommentPathFormatError": {
      return "Not a valid comment path. Comment paths must be a (possibly empty) list of dot seperated numbers, followed by a # and then the comment number.\n\nExamples: #2 or 1.2.3#4";
    }
    case "InvalidHeaderPathError": {
      return "Not a valid new header path. Either the parent header does not exist or it does not have enough subheaders.\n\nExample: To add header 1.2.3, parent header 1.2 must exist and must already have two subheaders to add the subheader number 3.";
    }
    case "InvalidCommentPathError": {
      return "Not a valid new comment path. Either the parent header does not exist or it does not have enough comments.\n\nExample: To add comment 1.2.3#4, parent header 1.2.3 must exist and must already have three comments to add comment #4.";
    }
    case "LabelAlreadyExistsError":
    case "SymbolAlreadyExistsError": {
      return "This token already exists as a label or math symbol.";
    }
    case "TypecodeNotAConstantError": {
      return "Not an active constant. A variable's typecode must always be an active constant.";
    }
    case "ExpectedActiveVariableError": {
      return "Not an active variable. Floating hypotheses can only declare the typecode of active variables.";
    }
    case "VariableTypecodeAlreadyDeclaredError": {
      return "The typecode of this variable has already been declared in a different floating hypothesis.";
    }
    case "InvalidMathSymbolError": {
      return "Not a valid math symbol. A math symbol token may consist of any combination of the 93 printable standard ascii characters other than space or $.";
    }
    case "TwiceDeclaredMathSymbolError": {
      return "Can't declare the same symbol twice.";
    }
    case "MmpStepRefNotALabelError": {
      return "Not the label of a previous theorem or axiom.";
    }
    case "TooManyHypothesesError": {
      return "Too many hypotheses for the provided theorem.";
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
