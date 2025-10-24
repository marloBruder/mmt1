use tauri::async_runtime::Mutex;

use crate::{
    editor::format,
    metamath::{
        export,
        mmp_parser::{
            self, MmpParserStage1, MmpParserStage2, MmpParserStage3, MmpParserStage3Success,
            MmpParserStage4, MmpStatement, UnifyLine,
        },
    },
    model::MetamathData,
    util::{self, StrIterToDelimiterSeperatedString},
    AppState, Error,
};

#[tauri::command]
pub async fn unify(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<Option<String>, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;
    let settings = &app_state.settings;

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

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data, None)?;

    let stage_6 = stage_5.next_stage(&stage_3_theorem, &stage_4_success, mm_data, settings)?;

    let mut result_text = String::new();
    let mut unify_line_iter = stage_5.unify_result.into_iter();

    let mut proof_added = false;

    for (&statement, (statement_type, _)) in stage_1_success
        .statements
        .iter()
        .zip(stage_2_success.statements.iter())
    {
        if matches!(statement_type, MmpStatement::ProofLine) {
            let mut unify_line = unify_line_iter.next().ok_or(Error::InternalLogicError)?;

            while unify_line.new_line {
                if !unify_line.deleted_line {
                    write_unify_line(&mut result_text, unify_line, 1, mm_data)?;
                }
                unify_line = unify_line_iter.next().ok_or(Error::InternalLogicError)?;
            }

            if !unify_line.deleted_line {
                write_unify_line(
                    &mut result_text,
                    unify_line,
                    util::new_lines_at_end_of_str(statement),
                    mm_data,
                )?;
            }
        } else if matches!(statement_type, MmpStatement::Proof) {
            if let Some(proof) = &stage_6.proof {
                write_proof(&mut result_text, proof)?;

                for _ in 0..std::cmp::min(util::new_lines_at_end_of_str(statement), 2) {
                    result_text.push('\n');
                }

                proof_added = true;
            }
        } else {
            result_text.push_str(statement);
        }
    }

    if let Some(proof) = &stage_6.proof {
        if !proof_added {
            result_text.push('\n');
            result_text.push('\n');
            write_proof(&mut result_text, proof)?;
            result_text.push('\n');
        }
    }

    format::format_mmp_file(&result_text)
}

fn write_unify_line(
    result_text: &mut String,
    unify_line: UnifyLine,
    new_lines_at_end_of_statement: u32,
    mm_data: &MetamathData,
) -> Result<(), Error> {
    if unify_line.advanced_unification {
        result_text.push('!');
    }
    if unify_line.is_hypothesis {
        result_text.push('h');
    }

    result_text.push_str(&unify_line.step_name);
    result_text.push(':');
    result_text.push_str(
        &unify_line
            .hypotheses
            .into_iter()
            .fold_to_delimiter_seperated_string(","),
    );
    result_text.push(':');
    result_text.push_str(&unify_line.step_ref);

    result_text.push(' ');

    result_text.push_str(
        &unify_line
            .parse_tree
            .map(|pt| {
                pt.to_expression(
                    &mm_data.optimized_data.symbol_number_mapping,
                    &mm_data.optimized_data.grammar,
                )
            })
            .transpose()?
            .unwrap_or(String::new()),
    );

    for _ in 0..std::cmp::min(new_lines_at_end_of_statement, 2) {
        result_text.push('\n');
    }

    Ok(())
}

fn write_proof(result_text: &mut String, proof: &str) -> Result<(), Error> {
    result_text.push_str("$=");

    if proof.starts_with('(') {
        let (label_str, step_str) = proof.split_once(')').ok_or(Error::InternalLogicError)?;

        export::write_text_wrapped(result_text, label_str, "  ");
        result_text.push_str(" ) ");
        export::write_text_wrapped_no_whitespace(result_text, step_str, "  ");
    } else {
        export::write_text_wrapped(result_text, proof, "  ");
    }

    Ok(())
}
