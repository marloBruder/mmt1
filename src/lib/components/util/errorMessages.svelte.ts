export function getErrorMessage(errorType: string): string {
  switch (errorType) {
    case "WhitespaceBeforeFirstTokenError": {
      return "Statements can't have trailing whitespace.\n\n(This error only shows before the first statement, because other lines with trailing whitespace continue the previous statement.)";
    }
    case "TooManyConstStatementsError": {
      return "You can only declare one constant statement per mmp file.\n\n(But you can declare multiple constants in one statement.)";
    }
    case "EmptyConstStatementError": {
      return "Empty constant statement.";
    }
  }
  return "";
}
