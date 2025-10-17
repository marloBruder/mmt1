use tauri::async_runtime::Mutex;

use crate::{
    metamath::mmp_parser::{
        self, calc_indention::calc_indention, MmpParserStage1, MmpParserStage2, MmpParserStage3,
        MmpParserStage3Success, MmpParserStage4, MmpStatement, UnifyLine,
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

    let stage_5 = stage_4_success.next_stage(&stage_2_success, &stage_3_theorem, mm_data)?;

    let indention_vec = calc_indention(&stage_5.unify_result)?.into_iter();

    let mut result_text = String::new();
    let mut unify_line_iter = stage_5
        .unify_result
        .into_iter()
        .zip(indention_vec.into_iter());

    for (&statement, (statement_type, _)) in stage_1_success
        .statements
        .iter()
        .zip(stage_2_success.statements.iter())
    {
        if matches!(statement_type, MmpStatement::ProofLine) {
            let (mut unify_line, mut indention) =
                unify_line_iter.next().ok_or(Error::InternalLogicError)?;

            while unify_line.new_line {
                if !unify_line.deleted_line {
                    write_unify_line(&mut result_text, unify_line, 1, indention, mm_data)?;
                }
                (unify_line, indention) =
                    unify_line_iter.next().ok_or(Error::InternalLogicError)?;
            }

            if !unify_line.deleted_line {
                write_unify_line(
                    &mut result_text,
                    unify_line,
                    util::new_lines_at_end_of_str(statement),
                    indention,
                    mm_data,
                )?;
            }
        } else {
            result_text.push_str(statement);
        }
    }

    Ok(Some(result_text))
}

fn write_unify_line(
    result_text: &mut String,
    unify_line: UnifyLine,
    new_lines_at_end_of_statement: u32,
    indention: u32,
    mm_data: &MetamathData,
) -> Result<(), Error> {
    let prefix_len = unify_line.advanced_unification as u32
        + unify_line.is_hypothesis as u32
        + unify_line.step_name.len() as u32
        + 1
        + unify_line
            .hypotheses
            .iter()
            .map(|hyp| hyp.len() + 1)
            .sum::<usize>() as u32
        - !unify_line.hypotheses.is_empty() as u32
        + 1
        + unify_line.step_ref.len() as u32;

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

    if prefix_len >= 20 + indention - 1 {
        result_text.push('\n');
        result_text.push_str(util::spaces(20));
        result_text.push_str(util::spaces(indention - 1));
    } else {
        result_text.push_str(util::spaces(20 - prefix_len));
        result_text.push_str(util::spaces(indention - 1));
    }

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

    for _ in 0..new_lines_at_end_of_statement {
        result_text.push('\n');
    }

    Ok(())
}
