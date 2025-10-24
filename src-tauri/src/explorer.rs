use std::u32;

use tauri::async_runtime::Mutex;

use crate::{
    model::{
        CommentListEntry, ConstantListEntry, DatabaseElement, FloatingHypothesisListEntry,
        HeaderListEntry, HeaderPath, ListEntry,
        Statement::{self},
        TheoremListData, VariableListEntry,
    },
    search,
    util::{self, StrIterToSpaceSeperatedString},
    AppState, Error,
};

// Returns a tuple (theorems, more), where theorems are theorem names that match the query
// and more is whether there exist more theorems that do as well.
// If only_ten is true, only ten theorems will be returned
#[tauri::command]
pub async fn quick_search(
    state: tauri::State<'_, Mutex<AppState>>,
    query: &str,
    only_ten: bool,
) -> Result<(Vec<String>, bool), Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let limit = if only_ten { 11 } else { u32::MAX };

    let mut theorems =
        search::find_theorem_labels(&metamath_data.database_header, query, limit, |_| true);

    let mut more = false;
    if only_ten && theorems.len() == 11 {
        more = true;
        theorems.pop();
    }

    Ok((theorems, more))
}

// page starts at 0
#[tauri::command]
pub async fn get_theorem_list(
    state: tauri::State<'_, Mutex<AppState>>,
    page: u32,
) -> Result<TheoremListData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;
    let mut curr_header_path: HeaderPath = HeaderPath::new();
    let mut curr_header_comment_amount: u32 = 0;
    let mut list: Vec<ListEntry> = Vec::new();

    for database_element in metamath_data.database_header.iter() {
        if theorem_amount >= (page + 1) * 100 {
            break;
        }

        match database_element {
            DatabaseElement::Header(header, depth) => {
                util::calc_next_header_path(&mut curr_header_path, depth)?;

                curr_header_comment_amount = 0;

                let header_path = curr_header_path.to_string();

                let description_parsed = metamath_data
                    .optimized_data
                    .header_data
                    .get(&header_path)
                    .ok_or(Error::InternalLogicError)?
                    .description_parsed
                    .clone();

                if page * 100 <= theorem_amount {
                    list.push(ListEntry::Header(HeaderListEntry {
                        header_path,
                        title: header.title.clone(),
                        description_parsed,
                    }));
                }
            }
            DatabaseElement::Statement(statement) => match statement {
                Statement::CommentStatement(comment) => {
                    curr_header_comment_amount += 1;
                    if page * 100 <= theorem_amount {
                        list.push(ListEntry::Comment(CommentListEntry {
                            comment_path: format!(
                                "{}#{}",
                                curr_header_path.to_string(),
                                curr_header_comment_amount
                            ),
                            text: comment.text.clone(),
                        }));
                    }
                }
                Statement::ConstantStatement(constants) => {
                    if page * 100 <= theorem_amount {
                        list.push(ListEntry::Constant(ConstantListEntry {
                            constants: constants
                                .iter()
                                .map(|c| &*c.symbol)
                                .fold_to_space_seperated_string(),
                        }));
                    }
                }
                Statement::VariableStatement(variables) => {
                    if page * 100 <= theorem_amount {
                        list.push(ListEntry::Variable(VariableListEntry {
                            variables: variables
                                .iter()
                                .map(|v| &*v.symbol)
                                .fold_to_space_seperated_string(),
                        }));
                    }
                }
                Statement::FloatingHypohesisStatement(floating_hypothesis) => {
                    if page * 100 <= theorem_amount {
                        list.push(ListEntry::FloatingHypohesis(FloatingHypothesisListEntry {
                            label: floating_hypothesis.label.clone(),
                            typecode: floating_hypothesis.typecode.clone(),
                            variable: floating_hypothesis.variable.clone(),
                        }));
                    }
                }
                Statement::TheoremStatement(theorem) => {
                    if page * 100 <= theorem_amount {
                        list.push(ListEntry::Theorem(theorem.to_theorem_list_entry(
                            (theorem_amount + 1) as u32,
                            &metamath_data.optimized_data,
                        )));
                    }
                    theorem_amount += 1;
                }
            },
        }
    }

    let page_amount =
        ((((metamath_data.optimized_data.theorem_amount as i32) - 1) / 100) + 1) as u32;

    Ok(TheoremListData {
        list,
        page_amount,
        theorem_amount: metamath_data.optimized_data.theorem_amount,
        page_limits: None,
    })
}

#[tauri::command]
pub async fn get_theorem_list_page_of_header(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;
    let mut curr_header_path: HeaderPath = HeaderPath::new();

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Header(_, depth) => {
                util::calc_next_header_path(&mut curr_header_path, depth)?;

                if curr_header_path == header_path {
                    break;
                }
            }
            DatabaseElement::Statement(Statement::TheoremStatement(_)) => {
                theorem_amount += 1;
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}

#[tauri::command]
pub async fn get_theorem_list_page_of_comment(
    state: tauri::State<'_, Mutex<AppState>>,
    header_path: HeaderPath,
    comment_i: u32,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;
    let mut curr_header_path: HeaderPath = HeaderPath::new();
    let mut curr_comment_i: u32 = 0;

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Header(_, depth) => {
                util::calc_next_header_path(&mut curr_header_path, depth)?;
                curr_comment_i = 0;

                if curr_header_path == header_path {
                    break;
                }
            }
            DatabaseElement::Statement(Statement::CommentStatement(_)) => {
                if curr_comment_i == comment_i && curr_header_path == header_path {
                    break;
                }

                curr_comment_i += 1;
            }
            DatabaseElement::Statement(Statement::TheoremStatement(_)) => {
                theorem_amount += 1;
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}

#[tauri::command]
pub async fn get_theorem_list_page_of_constant(
    state: tauri::State<'_, Mutex<AppState>>,
    any_constant: &str,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Statement(Statement::TheoremStatement(_)) => {
                theorem_amount += 1;
            }
            DatabaseElement::Statement(Statement::ConstantStatement(consts)) => {
                if consts.iter().any(|c| c.symbol == any_constant) {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}

#[tauri::command]
pub async fn get_theorem_list_page_of_variable(
    state: tauri::State<'_, Mutex<AppState>>,
    any_variable: &str,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Statement(Statement::TheoremStatement(_)) => {
                theorem_amount += 1;
            }
            DatabaseElement::Statement(Statement::VariableStatement(vars)) => {
                if vars.iter().any(|v| v.symbol == any_variable) {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}

#[tauri::command]
pub async fn get_theorem_list_page_of_floating_hypothesis(
    state: tauri::State<'_, Mutex<AppState>>,
    floating_hypothesis_label: &str,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Statement(Statement::TheoremStatement(_)) => {
                theorem_amount += 1;
            }
            DatabaseElement::Statement(Statement::FloatingHypohesisStatement(fh)) => {
                if fh.label == floating_hypothesis_label {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}

#[tauri::command]
pub async fn get_theorem_list_page_of_theorem(
    state: tauri::State<'_, Mutex<AppState>>,
    theorem_label: &str,
) -> Result<u32, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;

    for database_element in metamath_data.database_header.iter() {
        match database_element {
            DatabaseElement::Statement(Statement::TheoremStatement(t)) => {
                if t.label == theorem_label {
                    break;
                }

                theorem_amount += 1;
            }
            _ => {}
        }
    }

    Ok(theorem_amount / 100)
}
