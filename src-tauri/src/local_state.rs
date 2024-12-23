use tauri::async_runtime::Mutex;

use crate::{
    model::{Hypothesis, InProgressTheorem, MetamathData, Theorem},
    AppState,
};

#[tauri::command]
pub async fn get_theorem_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<Theorem, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        for theorem in &mm_data.theorems {
            if theorem.name == name {
                return Ok(theorem.clone());
            }
        }
    }

    Err(())
}

#[tauri::command]
pub async fn get_theorem_names_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let mut names: Vec<String> = Vec::new();
        for theorem in &mm_data.theorems {
            names.push(theorem.name.clone());
        }
        return Ok(names);
    }

    Err(())
}

#[tauri::command]
pub async fn get_in_progress_theorem_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<InProgressTheorem, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        for in_progress_theorem in &mm_data.in_progress_theorems {
            if in_progress_theorem.name == name {
                return Ok(in_progress_theorem.clone());
            }
        }
    }

    Err(())
}

#[tauri::command]
pub async fn get_in_progress_theorem_names_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        let mut names: Vec<String> = Vec::new();
        for in_progress_theorem in &mm_data.in_progress_theorems {
            names.push(in_progress_theorem.name.clone());
        }
        return Ok(names);
    }

    Err(())
}

impl MetamathData {
    pub fn add_in_progress_theorem(&mut self, name: &str, text: &str) {
        self.in_progress_theorems.push(InProgressTheorem {
            name: name.to_string(),
            text: text.to_string(),
        })
    }

    pub fn set_in_progress_theorem_name(&mut self, old_name: &str, new_name: &str) {
        for in_progress_theorem in &mut self.in_progress_theorems {
            if in_progress_theorem.name == old_name {
                in_progress_theorem.name = new_name.to_string();
            }
        }
    }

    pub fn set_in_progress_theorem_text(&mut self, name: &str, text: &str) {
        for in_progress_theorem in &mut self.in_progress_theorems {
            if in_progress_theorem.name == name {
                in_progress_theorem.text = text.to_string();
            }
        }
    }

    pub fn delete_in_progress_theorem(&mut self, name: &str) {
        for (index, in_progress_theorem) in (&self.in_progress_theorems).iter().enumerate() {
            if in_progress_theorem.name == name {
                self.in_progress_theorems.remove(index);
                return;
            }
        }
    }

    pub fn add_theorem(
        &mut self,
        name: &str,
        description: &str,
        disjoints: &Vec<String>,
        hypotheses: &Vec<Hypothesis>,
        assertion: &str,
        proof: Option<&str>,
    ) {
        self.theorems.push(Theorem {
            name: name.to_string(),
            description: description.to_string(),
            disjoints: disjoints.clone(),
            hypotheses: hypotheses.clone(),
            assertion: assertion.to_string(),
            proof: proof.map(|s| s.to_string()),
        })
    }
}
