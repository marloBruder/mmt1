import monaco from "$lib/monaco/monaco";

export function getMmpFileErrorMessage(errorType: string): string {
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
    // case "MissingCommentPathError":
    // case "TooManyCommentPathTokensError": {
    //   return "$comment statements must only be followed by exactly one token: The path of the comment.\n\nExample: $comment 3.4.2#5 (The fifth comment under header 3.4.2)";
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
    case "TooFewLocateAfterVarTokensError":
    case "TooManyLocateAfterVarTokensError": {
      return "$locateaftervar statements must be followed by exactly one token: The variable which statement the content of the mmp file should be located after.";
    }
    case "TooFewLocateAfterHeaderTokensError":
    case "TooManyLocateAfterHeaderTokensError": {
      return "$locateafterheader statements must be followed by exactly one token: The header path of the header that the content of the mmp file should be located after.";
    }
    case "TooFewLocateAfterCommentTokensError":
    case "TooManyLocateAfterCommentTokensError": {
      return "$locateaftercomment statements must be followed by exactly one token: The comment path of the comment that the content of the mmp file should be located after.";
    }
    case "TooManyLocateAfterStartTokensError": {
      return "$locateafterstart statements should not be followed by any tokens.";
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
      return "$c statement should not be here. $c statements are only allowed when describing a constant statement and can not appear alongside $header, $axiom, $theorem, $v or $f statements.";
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
      return "Locate after statement should not be here. Locate after statements may not appear alongside $header statements, as their location in the database is determined by their header path.";
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
    case "InvalidNewHeaderPathError": {
      return "Not a valid new header path. Either the parent header does not exist or it does not have enough subheaders.\n\nExample: To add header 1.2.3, parent header 1.2 must exist and must already have two subheaders to add the subheader number 3.";
    }
    case "InvalidHeaderPathError": {
      return "Not a valid existing header path.";
    }
    case "InvalidCommentPathError": {
      return "Not a valid existing comment path. Either the parent header does not exist or it does not have enough comments.";
    }
    // case "InvalidNewCommentPathError": {
    //   return "Not a valid new comment path. Either the parent header does not exist or it does not have enough comments.\n\nExample: To add comment 1.2.3#4, parent header 1.2.3 must exist and must already have three comments to add comment #4.";
    // }
    case "LabelAlreadyExistsError":
    case "SymbolAlreadyExistsError": {
      return "This token already exists as a label or math symbol.";
    }
    case "NonTheoremLabelAlreadyExistsError": {
      return "This token already exists as a label or math symbol, which is not a theorem label.";
    }
    case "NonFloatHypLabelAlreadyExistsError": {
      return "This token already exists as a label or math symbol, which is not a floating hypothesis label.";
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
    case "SyntaxTheoremUsedError": {
      return "Can't use syntax theorems in mmp files.";
    }
    case "HypothesisWithHypsError": {
      return "Hypotheses lines can't have hypotheses.";
    }
    case "MultipleAllowIncompleteError": {
      return "There should be at most one $allowincomplete statement per mmp file.";
    }
    case "TokensAfterAllowIncompleteError": {
      return "$allowincomplete statements should not be followed by any tokens.";
    }
    case "DiscouragedTheoremUsedError": {
      return "The theorem referenced is discouraged. Use $allowdiscouraged to allow discouraged theorems in this proof.";
    }
    case "IncompleteTheoremUsedError": {
      return "The theorem referenced is incomplete or has an incomplete theorem in it's dependency tree. Use $allowdincomplete to allow incomplete theorems in this proof.";
    }
    case "InvalidTypecodeError": {
      return "Not a valid typecode. Typecodes must be registered using $j syntax comments.";
    }
    case "InvalidMmpStepForAxiomError": {
      return "When adding an axiom to the database, there should only be hypotheses or qed mmp-steps.";
    }
    case "AxiomStepWithHypError": {
      return "No mmp-step alongside an $axiom statement should have hypotheses.";
    }
    case "AxiomQedStepWithRefError": {
      return "When adding an axiom to the database, the step ref field of the qed step must be empty.";
    }
    case "SyntaxAxiomWithHypothesesError": {
      return "Syntax axioms cannot have hypotheses.";
    }
    case "AxiomWithWorkVariableError": {
      return "Can't use work variables when adding an axiom.";
    }
    case "MultipleProofStatementsError": {
      return "There can only be one $= statement per mmp file.";
    }
    case "HeaderPathLengthGreater4Error": {
      return "Header paths may only go 4 levels deep.";
    }
    case "NotAConstantError": {
      return "Not a constant symbol.";
    }
    case "NotAVariableError": {
      return "Not a variable symbol.";
    }
    case "NotAValidLabelError": {
      return "Not a floating hypothesis or theorem label.";
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

export function getAddToDatabaseErrorMessage(errorType: string): string {
  switch (errorType) {
    case "CantAddToDatabaseError": {
      return "Can't add a statement to the database while the editor still contains errors.";
    }
    case "UnfinishedTheoremError": {
      return "The theorem or axiom you are trying to add is not yet finished. Perhaps a hypothesis or the qed step is missing their expression or the proof is simply not yet finished.";
    }
    case "MmpFileEmptyError": {
      return "You cannot add empty mmp files to the database.";
    }
    case "DatabaseHasChangedError": {
      return "The opened mm file has changed since it was loaded. Please reload the database to add statements to the database again.";
    }
    case "AddingToInnerScopeError": {
      return "The location you are trying to add to is inbetween two statements sharing a scope. mmt1 is not yet cappable of handling this case. Please add the statement manually.";
    }
    case "FileReadError": {
      return "Cannot read mm file. Please make sure that the file was not moved or deleted.";
    }
  }

  return "You should not be seeing this error message. Please post a Github issue with your mmp file.";
}

export function getMmFileErrorMessage(errorType: string): string {
  switch (errorType) {
    case "FileReadError": {
      return "Could not read mm file. Make sure you have the necessary permissions";
    }
    case "InvalidCharactersError": {
      return "The mm file contains non-ascii characters.";
    }
    case "TokenOutsideStatementError": {
      return "There is a token which does not belong to any statement.";
    }
    case "UnclosedCommentError": {
      return "There is an unclosed comment.";
    }
    case "AdditionalInfoCommentFormatError": {
      return "An additional information comment has an invalid format.";
    }
    case "InvalidColorCodeError": {
      return "A color code definied by varcolorcode or altvarcolorcode is not a valid color code string.";
    }
    case "TypesettingFormatError": {
      return "A statement within the typesetting comment has an invalid format.";
    }
    case "InvalidHeaderDepthError": {
      return "A header was an invalid header depth. mmt1 requires that all headers are exactly one depth below their parent header.";
    }
    case "UnclosedHeaderError": {
      return "There is an unclosed header. (A comment that starts with e.g. #### but does not have a another token starting with ####)";
    }
    case "ClosedUnopenedScopeError": {
      return "There is a closing scope statement in the outer most scope.";
    }
    case "ConstStatementScopeError": {
      return "There is a constant statment that is not within the outer most scope.";
    }
    case "InvalidSymbolError": {
      return "There is a symbol declared that does not follow the rules for what characters symbols can use.";
    }
    case "TwiceDeclaredConstError": {
      return "A constant is declared which symbol was already previously declared.";
    }
    case "EmptyConstStatementError": {
      return "There is an empty constant statement.";
    }
    case "TwiceDeclaredVarError": {
      return "A variable is declared which symbol was already previously declared.";
    }
    case "EmptyVarStatementError": {
      return "There is an empty variable statement.";
    }
    case "FloatHypStatementFormatError": {
      return "There is a floating hypothesis statement which does not have exactly two non comment tokens.";
    }
    case "MissingLabelError": {
      return "A statement is missing it's label.";
    }
    case "FloatHypTypecodeError": {
      return "The typecode declared in a floating hypothesis is not an active constant.";
    }
    case "FloatHypVariableError": {
      return "The variable declared in a floating hypothesis is not an active variable.";
    }
    case "VarTypeDeclaredTwiceError": {
      return "The typecode of a variable is declared twice.";
    }
    case "VarDeclaredMultipleTypesError": {
      return "The typecode of a variable is declared as a typecode that differs from the typecode that was previously declared.";
    }
    case "NonSymbolInExpressionError": {
      return "There is a symbol in an expression that is not an active constant or variable.";
    }
    case "ZeroOrOneSymbolDisjError": {
      return "There is a disjoint variable statement with less than two non-comment tokens.";
    }
    case "NonVarInDisjError": {
      return "There is a disjoint variable statement with a non-variable token.";
    }
    case "UnclosedScopeError": {
      return "There is an unclosed scope statement.";
    }
    case "InvalidLabelError": {
      return "There is a label declared that does not follow the rules for what characters labels can use.";
    }
    case "InvalidProofError": {
      return "There is a proof that could not be successfully verified.";
    }
    case "TwiceDeclaredLabelError": {
      return "There is a label which has previously been used as a label or symbol.";
    }
  }

  return "You should not be seeing this error message. Please post a Github issue with a link to your mm file.";
}
