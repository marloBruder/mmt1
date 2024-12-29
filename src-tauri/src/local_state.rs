use crate::{
    metamath::{self, calc_theorem_page_data},
    model::{
        Constant, FloatingHypohesis, Hypothesis, InProgressTheorem, MetamathData, Theorem,
        TheoremPageData, Variable,
    },
    AppState,
};
use tauri::async_runtime::Mutex;

#[tauri::command]
pub async fn get_constants_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Constant>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        return Ok(mm_data.constants.clone());
    }

    Err(())
}

#[tauri::command]
pub async fn get_variables_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Variable>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        return Ok(mm_data.variables.clone());
    }

    Err(())
}

#[tauri::command]
pub async fn get_floating_hypotheses_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<FloatingHypohesis>, ()> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        return Ok(mm_data.floating_hypotheses.clone());
    }

    Err(())
}

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    name: &str,
) -> Result<TheoremPageData, metamath::Error> {
    let app_state = state.lock().await;

    if let Some(ref mm_data) = app_state.metamath_data {
        for theorem in &mm_data.theorems {
            if theorem.name == name {
                return calc_theorem_page_data(&theorem, mm_data);
            }
        }
    }

    Err(metamath::Error::NotFoundError)
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
    pub fn set_constants(&mut self, symbols: &Vec<&str>) {
        self.constants = Vec::new();
        for symbol in symbols {
            self.constants.push(Constant {
                symbol: symbol.to_string(),
            })
        }
    }

    pub fn set_variables(&mut self, symbols: &Vec<&str>) {
        self.variables = Vec::new();
        for symbol in symbols {
            self.variables.push(Variable {
                symbol: symbol.to_string(),
            })
        }
    }

    pub fn set_floating_hypotheses(&mut self, floating_hypotheses: &Vec<FloatingHypohesis>) {
        self.floating_hypotheses = floating_hypotheses.clone();
    }

    // pub fn set_floating_hypotheses(
    //     &mut self,
    //     labels: &Vec<&str>,
    //     typecodes: &Vec<&str>,
    //     variables: &Vec<&str>,
    // ) {
    //     self.floating_hypotheses = Vec::new();
    //     for index in 0..labels.len() {
    //         self.floating_hypotheses.push(FloatingHypohesis {
    //             label: labels[index].to_string(),
    //             typecode: typecodes[index].to_string(),
    //             variable: variables[index].to_string(),
    //         })
    //     }
    // }

    pub fn get_theorem_by_name(&self, name: &str) -> Result<&Theorem, metamath::Error> {
        for theorem in &self.theorems {
            if theorem.name == name {
                return Ok(&theorem);
            }
        }

        Err(metamath::Error::NotFoundError)
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
}
