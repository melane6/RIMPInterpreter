// The error handler module contains the error handling logic for the RIMPInterpreter.
// The parser and lexer modules can return errors such as syntax errors, and semantic errors.
// Parser Error come in the form of a ParseError, and Lexer errors come in the form of a LexError.
// To correctly display ParseErrors:
//     - Use the usize field to determine the token of the error
//     - Traverse the token stream to find the line and column of the error
//     - Display the line and column of the error to the user to indicate where the error is
//     - Display an appropriate error message for the ParseErrorMessage obtained from the ParseError
// To correctly display LexErrors:
//     - Lexing errors are generic and can be displayed as is
//     - Highlight all text that was run to indicate the error
//
// E.g.
//    For the ParseError:
//   ParseError {
//      token: 5,
//      message: ParseErrorMessage::ExpectedToken(Token::FnId("fn".to_string())), E
