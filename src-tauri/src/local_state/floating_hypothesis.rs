use tauri::async_runtime::Mutex;

use crate::{
    model::{FloatingHypothesis, FloatingHypothesisPageData, MetamathData},
    AppState, Error,
};

#[tauri::command]
pub async fn get_floating_hypothesis_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
) -> Result<FloatingHypothesisPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(FloatingHypothesisPageData {
        floating_hypothesis: metamath_data
            .optimized_data
            .floating_hypotheses
            .iter()
            .find(|fh| fh.label == label)
            .map(|fh| fh.clone())
            .ok_or(Error::NotFoundError)?,
    })
}

#[tauri::command]
pub async fn get_floating_hypotheses_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<FloatingHypothesis>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data.optimized_data.floating_hypotheses.clone())
}

pub fn get_floating_hypothesis_by_label<'a>(
    metamath_data: &'a MetamathData,
    label: &str,
) -> Option<&'a FloatingHypothesis> {
    for floating_hypothesis in &metamath_data.optimized_data.floating_hypotheses {
        if floating_hypothesis.label == label {
            return Some(floating_hypothesis);
        }
    }
    return None;
}

// pub fn set_floating_hypotheses_local(
//     metamath_data: &mut MetamathData,
//     floating_hypotheses: &Vec<FloatingHypohesis>,
// ) {
//     metamath_data.optimized_data.floating_hypotheses = floating_hypotheses.clone();
// }
