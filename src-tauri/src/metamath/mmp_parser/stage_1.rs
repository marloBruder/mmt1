use crate::{editor::on_edit::DetailedError, Error};

use super::{MmpParserStage0, MmpParserStage1, MmpParserStage1Fail, MmpParserStage1Success};

pub fn stage_1(stage_0: MmpParserStage0) -> Result<MmpParserStage1, Error> {
    let mut statements = Vec::new();

    let mut text_i: usize = 0;
    let text_bytes = stage_0.text.as_bytes();

    let mut number_of_lines_before_first_statement: u32 = 1;
    let mut last_line_length: u32 = 0;

    while text_bytes
        .get(text_i)
        .is_some_and(|c| c.is_ascii_whitespace())
    {
        last_line_length += 1;

        if text_bytes.get(text_i).is_some_and(|c| *c == b'\n') {
            number_of_lines_before_first_statement += 1;
            last_line_length = 0;
        }

        text_i += 1;
    }

    if text_i != 0 && text_bytes.get(text_i - 1).is_some_and(|c| *c != b'\n') {
        return Ok(MmpParserStage1::Fail(MmpParserStage1Fail {
            error: DetailedError {
                error_type: Error::WhitespaceBeforeFirstTokenError,
                start_line_number: number_of_lines_before_first_statement,
                start_column: 1,
                end_line_number: number_of_lines_before_first_statement,
                end_column: last_line_length + 1,
            },
        }));
    }

    let mut statement_start = text_i;
    text_i += 1;

    while let Some(&char) = text_bytes.get(text_i) {
        if !char.is_ascii_whitespace() && text_bytes.get(text_i - 1).is_some_and(|c| *c == b'\n') {
            statements.push(
                stage_0
                    .text
                    .get(statement_start..text_i)
                    .ok_or(Error::InternalLogicError)?,
            );
            statement_start = text_i;
        }

        text_i += 1;
    }

    statements.push(
        stage_0
            .text
            .get(statement_start..text_i)
            .ok_or(Error::InternalLogicError)?,
    );

    Ok(MmpParserStage1::Success(MmpParserStage1Success {
        number_of_lines_before_first_statement,
        statements,
    }))
}
