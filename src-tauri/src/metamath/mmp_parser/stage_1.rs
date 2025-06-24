use crate::{editor::on_edit::DetailedError, Error};

use super::{MmpParserStage0, MmpParserStage1, MmpParserStage1Fail, MmpParserStage1Success};

pub fn stage_1<'a>(stage_0: &MmpParserStage0<'a>) -> Result<MmpParserStage1<'a>, Error> {
    if !stage_0.text.is_ascii() {
        let text_end_pos = text_end_pos(stage_0.text);

        return Ok(MmpParserStage1::Fail(MmpParserStage1Fail {
            error: DetailedError {
                error_type: Error::NonAsciiSymbolError,
                start_line_number: 1,
                start_column: 1,
                end_line_number: text_end_pos.0,
                end_column: text_end_pos.1 + 1,
            },
        }));
    }

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

    if text_i != 0
        && text_i != text_bytes.len()
        && text_bytes.get(text_i - 1).is_some_and(|c| *c != b'\n')
    {
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

    if statement_start < text_bytes.len() {
        statements.push(
            stage_0
                .text
                .get(statement_start..text_i)
                .ok_or(Error::InternalLogicError)?,
        );
    }

    Ok(MmpParserStage1::Success(MmpParserStage1Success {
        number_of_lines_before_first_statement,
        statements,
    }))
}

fn text_end_pos(text: &str) -> (u32, u32) {
    let mut column = 0;
    let mut line = 1;

    for char in text.chars() {
        if char == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }

    (line, column)
}
