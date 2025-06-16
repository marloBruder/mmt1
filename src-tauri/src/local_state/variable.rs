use tauri::async_runtime::Mutex;

use crate::{
    model::{DatabaseElement, Statement, Variable, VariablesPageData},
    util::ForEachWhile,
    AppState, Error,
};

#[tauri::command]
pub async fn get_variables_local(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Variable>, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    Ok(metamath_data
        .database_header
        .variable_iter()
        .map(|v| v.clone())
        .collect())
}

// Returns a vec of tuples (a, b), where a is a variable and b is it's typecode (as a String)
#[tauri::command]
pub async fn get_variable_statement_local(
    state: tauri::State<'_, Mutex<AppState>>,
    any_variable: &str,
) -> Result<VariablesPageData, Error> {
    let app_state = state.lock().await;
    let metamath_data = app_state.metamath_data.as_ref().ok_or(Error::NoMmDbError)?;

    let mut db_iter = metamath_data.database_header.iter();

    let mut variables: Vec<(Variable, String)> = db_iter
        .find_map(|db_element| match db_element {
            DatabaseElement::Statement(s) => match s {
                Statement::VariableStatement(vars) => {
                    if vars.iter().any(|v| v.symbol == any_variable) {
                        Some(vars.iter().map(|v| (v.clone(), String::new())).collect())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        })
        .ok_or(Error::NotFoundError)?;

    let mut typecodes_found = 0;

    db_iter.for_each_while(|db_element| {
        match db_element {
            DatabaseElement::Statement(s) => match s {
                Statement::FloatingHypohesisStatement(fh) => {
                    variables
                        .iter_mut()
                        .find(|(v, _)| v.symbol == fh.variable)
                        .map(|tuple| {
                            typecodes_found += 1;
                            tuple.1 = fh.typecode.clone();
                        });
                }
                _ => {}
            },
            _ => {}
        }

        // only continue if there are still typecodes to be found
        typecodes_found != variables.len()
    });

    Ok(VariablesPageData { variables })
}

// pub fn set_variables_local(metamath_data: &mut MetamathData, symbols: &Vec<&str>) {
//     metamath_data.variables = Vec::new();
//     for symbol in symbols {
//         metamath_data.variables.push(Variable {
//             symbol: symbol.to_string(),
//         })
//     }
// }
