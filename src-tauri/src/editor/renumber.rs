use tauri::async_runtime::Mutex;

use crate::{
    editor::format,
    metamath::mmp_parser::{
        self, MmpParserStage1, MmpParserStage2, MmpParserStage3, MmpParserStage3Success,
        MmpParserStage4, MmpStatement,
    },
    util::{self, StrIterToDelimiterSeperatedString},
    AppState, Error,
};

#[tauri::command]
pub async fn renumber(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Option<String>, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let stage_0 = mmp_parser::new(text);

    let MmpParserStage1::Success(stage_1_success) = stage_0.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage2::Success(stage_2_success) = stage_1_success.next_stage()? else {
        return Ok(None);
    };

    let MmpParserStage3::Success(MmpParserStage3Success::Theorem(stage_3_theorem)) =
        stage_2_success.next_stage(&stage_1_success, mm_data)?
    else {
        return Ok(None);
    };

    let MmpParserStage4::Success(stage_4_success) =
        stage_3_theorem.next_stage(&stage_1_success, &stage_2_success, mm_data)?
    else {
        return Ok(None);
    };

    let mut result_text = String::new();

    for _ in 0..(stage_1_success.number_of_lines_before_first_statement - 1) {
        result_text.push('\n');
    }

    let mut proof_line_i = 0;

    for (&statement, (statement_type, _)) in stage_1_success
        .statements
        .iter()
        .zip(stage_2_success.statements.iter())
    {
        if matches!(statement_type, MmpStatement::ProofLine) {
            let proof_line = stage_2_success
                .proof_lines
                .get(proof_line_i)
                .ok_or(Error::InternalLogicError)?;
            let proof_line_parsed = stage_4_success
                .proof_lines_parsed
                .get(proof_line_i)
                .ok_or(Error::InternalLogicError)?;

            if proof_line.advanced_unification {
                result_text.push('!');
            }
            if proof_line.is_hypothesis {
                result_text.push('h');
            }

            if proof_line.step_name == "qed" {
                result_text.push_str("qed");
            } else {
                result_text.push_str(&(proof_line_i + 1).to_string());
            }
            result_text.push(':');

            if !proof_line.hypotheses.is_empty() {
                result_text.push_str(
                    &proof_line_parsed
                        .hypotheses_parsed
                        .iter()
                        .map(|hyp_parsed| match hyp_parsed {
                            Some(hyp) => (hyp + 1).to_string(),
                            None => "?".to_string(),
                        })
                        .fold_to_delimiter_seperated_string(","),
                );
            }

            result_text.push(':');
            result_text.push_str(proof_line.step_ref);

            // should contain some whitespace at it's start
            result_text.push_str(proof_line.expression.trim_ascii_end());

            for _ in 0..util::new_lines_at_end_of_str(statement) {
                result_text.push('\n');
            }

            proof_line_i += 1;
        } else {
            result_text.push_str(statement);
        }
    }

    format::format_mmp_file(&result_text)
}
