use tauri::async_runtime::Mutex;

use crate::{
    metamath,
    model::{
        CommentListEntry, ConstantListEntry, DatabaseElement, FloatingHypothesisListEntry,
        HeaderListEntry, HeaderPath, ListEntry,
        Statement::{self},
        TheoremListData, TheoremPageData, VariableListEntry,
    },
    util::{self, StrIterToSpaceSeperatedString},
    AppState, Error,
};

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
    show_all: bool,
) -> Result<TheoremPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    metamath::calc_theorem_page_data(label, metamath_data, show_all)
}

// page starts at 0
#[tauri::command]
pub async fn get_theorem_list_local(
    state: tauri::State<'_, Mutex<AppState>>,
    page: u32,
) -> Result<TheoremListData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut theorem_amount: u32 = 0;
    let mut curr_header_path: HeaderPath = HeaderPath::new();
    let mut curr_header_comment_amount: u32 = 0;
    let mut list: Vec<ListEntry> = Vec::new();

    metamath_data
        .database_header
        .iter()
        .try_for_each(|database_element| {
            if theorem_amount >= (page + 1) * 100 {
                return Ok(());
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
            };
            Ok::<(), Error>(())
        })?;

    let page_amount =
        ((((metamath_data.optimized_data.theorem_amount as i32) - 1) / 100) + 1) as u32;

    Ok(TheoremListData {
        list,
        page_amount,
        page_limits: None,
    })
}
