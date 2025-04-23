use tauri::async_runtime::Mutex;

use crate::{
    metamath,
    model::{
        DatabaseElement, HeaderListEntry, HeaderPath, Hypothesis, ListEntry, MetamathData,
        Statement::{self, *},
        Theorem, TheoremListData, TheoremPageData, TheoremPath,
    },
    AppState, Error,
};

#[tauri::command]
pub async fn get_theorem_page_data_local(
    state: tauri::State<'_, Mutex<AppState>>,
    label: &str,
) -> Result<TheoremPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    metamath::calc_theorem_page_data(label, metamath_data)
}

pub fn get_theorem_insert_position(
    metamath_data: &MetamathData,
    position_name: &str,
) -> Result<TheoremPath, Error> {
    if position_name.contains(' ') {
        // Safe unwrap because of the prior condition
        let (_, header_title) = position_name.split_once(' ').unwrap();
        let header_path_res = metamath_data
            .database_header
            .calc_header_path_by_title(header_title);

        if let Some(header_path) = header_path_res {
            return Ok(TheoremPath {
                header_path,
                theorem_index: 0,
            });
        } else {
            return Err(Error::NotFoundError);
        }
    } else {
        let theorem_path_res = metamath_data
            .database_header
            .calc_theorem_path_by_label(position_name);

        if let Some(mut theorem_path) = theorem_path_res {
            theorem_path.theorem_index += 1;
            return Ok(theorem_path);
        } else {
            return Err(Error::NotFoundError);
        }
    }
}

pub fn add_theorem_local(
    metamath_data: &mut MetamathData,
    label: &str,
    description: &str,
    distincts: &Vec<String>,
    hypotheses: &Vec<Hypothesis>,
    assertion: &str,
    proof: Option<&str>,
    insert_path: &TheoremPath,
) -> Result<(), Error> {
    let header = insert_path
        .header_path
        .resolve_mut(&mut metamath_data.database_header)
        .ok_or(Error::NotFoundError)?;

    if header.content.len() < insert_path.theorem_index {
        return Err(Error::NotFoundError);
    }

    header.content.insert(
        insert_path.theorem_index,
        TheoremStatement(Theorem {
            label: label.to_string(),
            description: description.to_string(),
            distincts: distincts.clone(),
            hypotheses: hypotheses.clone(),
            assertion: assertion.to_string(),
            proof: proof.map(|s| s.to_string()),
        }),
    );

    Ok(())
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
    let mut curr_header_path: HeaderPath = HeaderPath { path: Vec::new() };
    let mut list: Vec<ListEntry> = Vec::new();

    metamath_data
        .database_header
        .iter()
        .try_for_each(|database_element| {
            match database_element {
                DatabaseElement::Header(header, depth) => {
                    // Calc next header path (only if there are still theorems ahead)
                    if theorem_amount < (page + 1) * 100 {
                        if depth > curr_header_path.path.len() as u32 {
                            curr_header_path.path.push(0);
                        } else if depth == curr_header_path.path.len() as u32 {
                            *curr_header_path
                                .path
                                .last_mut()
                                .ok_or(Error::InternalLogicError)? += 1;
                        } else if depth < curr_header_path.path.len() as u32 {
                            while depth < curr_header_path.path.len() as u32 {
                                curr_header_path.path.pop();
                            }
                            *curr_header_path
                                .path
                                .last_mut()
                                .ok_or(Error::InternalLogicError)? += 1;
                        }
                    }

                    if page * 100 <= theorem_amount && theorem_amount < (page + 1) * 100 {
                        list.push(ListEntry::Header(HeaderListEntry {
                            header_path: curr_header_path.to_string(),
                            title: header.title.clone(),
                        }));
                    }
                }
                DatabaseElement::Statement(statement) => match statement {
                    Statement::TheoremStatement(theorem) => {
                        if page * 100 <= theorem_amount && theorem_amount < (page + 1) * 100 {
                            list.push(ListEntry::Theorem(
                                theorem.to_theorem_list_entry((theorem_amount as u32) + 1),
                            ));
                        }
                        theorem_amount += 1;
                    }
                    _ => {}
                },
            };
            Ok::<(), Error>(())
        })?;

    let page_amount = ((((theorem_amount as i32) - 1) / 100) + 1) as u32;

    Ok(TheoremListData { list, page_amount })
}
