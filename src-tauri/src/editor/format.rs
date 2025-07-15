use tauri::async_runtime::Mutex;

use crate::{
    metamath::mmp_parser::{
        self, MmpParserStage1, MmpParserStage2, MmpParserStage3, MmpParserStage3Success,
        MmpParserStage4, MmpStatement,
    },
    util, AppState, Error,
};

#[tauri::command]
pub async fn format(
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

    let MmpParserStage4::Success(_) =
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
            let indention = *stage_3_theorem
                .indention
                .get(proof_line_i)
                .ok_or(Error::InternalLogicError)?;
            proof_line_i += 1;

            if proof_line.advanced_unification {
                result_text.push('!');
            }
            if proof_line.is_hypothesis {
                result_text.push('h');
            }

            result_text.push_str(proof_line.step_name);
            result_text.push(':');
            result_text.push_str(proof_line.hypotheses);
            result_text.push(':');
            result_text.push_str(proof_line.step_ref);

            let prefix_len = statement
                .split_ascii_whitespace()
                .next()
                .ok_or(Error::InternalLogicError)?
                .len() as u32;
            if prefix_len >= 20 + indention - 1 {
                result_text.push('\n');
                result_text.push_str(util::spaces(20));
                result_text.push_str(util::spaces(indention - 1));
            } else {
                result_text.push_str(util::spaces(indention - 1 + 20 - prefix_len));
            }

            result_text.push_str(&util::str_to_space_seperated_string(proof_line.expression));

            for _ in 0..util::new_lines_at_end_of_str(statement) {
                result_text.push('\n');
            }
        } else {
            result_text.push_str(statement);
        }
    }

    Ok(Some(result_text))
}
