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
    case "TooManyTheoremLabelTokensError": {
      return "$theorem statements must only be followed by a single token: The label of the theorem.";
    }
  }

  return "You should not be seeing this error message";
}
