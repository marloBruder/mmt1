use crate::{
    model::{self, Hypothesis, Theorem, TheoremPageData},
    util, AppState, Error,
};
use tauri::async_runtime::Mutex;

use super::unify::{self, MmpLabel};

#[tauri::command]
pub async fn on_edit(
    state: tauri::State<'_, Mutex<AppState>>,
    text: &str,
) -> Result<TheoremPageData, Error> {
    let app_state = state.lock().await;
    let mm_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let (_, statement_strs) = unify::text_to_statement_strs(text)?;
    let mmp_structured_info_for_unify =
        unify::statement_strs_to_mmp_info_structured_for_unify(&statement_strs, mm_data)?;

    let Some(MmpLabel::Theorem(label)) = mmp_structured_info_for_unify.label else {
        return Err(Error::MissingLabelError);
    };
    let label = label.to_string();

    let mut proof_lines: Vec<model::ProofLine> = Vec::new();

    let mut assertion = String::new();
    let mut hypotheses: Vec<Hypothesis> = Vec::new();

    for proof_line in &mmp_structured_info_for_unify.proof_lines {
        proof_lines.push(model::ProofLine {
            indention: 0,
            assertion: util::str_to_space_seperated_string(proof_line.expression),
            hypotheses: proof_line
                .hypotheses_parsed
                .iter()
                .map(|hyp| match hyp {
                    Some(num) => *num as i32 + 1,
                    None => 0,
                })
                .collect(),
            reference: proof_line.step_ref.to_string(),
        });

        if proof_line.step_name == "qed" {
            assertion = util::str_to_space_seperated_string(proof_line.expression);
        }
        if proof_line.is_hypothesis {
            hypotheses.push(Hypothesis {
                label: proof_line.step_ref.to_string(),
                expression: util::str_to_space_seperated_string(proof_line.expression),
            })
        }
    }

    let theorem = Theorem {
        label,
        description: mmp_structured_info_for_unify
            .comments
            .first()
            .unwrap_or(&"")
            .to_string(),
        assertion,
        distincts: mmp_structured_info_for_unify
            .distinct_vars
            .iter()
            .map(|d| util::str_to_space_seperated_string(d))
            .collect(),
        hypotheses,
        proof: Some("Proof not yet finished".to_string()),
    };

    Ok(TheoremPageData {
        theorem,
        theorem_number: 0,
        proof_lines,
        last_theorem_label: None,
        next_theorem_label: None,
    })
}
