use super::Error;
use crate::{
    model::{Hypothesis, Theorem},
    AppState,
};
use futures::TryStreamExt;
use sqlx::Row;
use tauri::async_runtime::Mutex;

pub async fn get_theorems(
    state: &tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Theorem>, Error> {
    let mut app_state = state.lock().await;

    let mut result = Vec::new();

    if let Some(ref mut conn) = app_state.db_conn {
        let mut rows = sqlx::query(sql::THEOREMS_GET).fetch(conn);

        while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
            let name: String = row.try_get("name").or(Err(Error::SqlError))?;
            let description: String = row.try_get("description").or(Err(Error::SqlError))?;
            let disjoints_rep: String = row.try_get("disjoints").or(Err(Error::SqlError))?;
            let hypotheses_rep: String = row.try_get("hypotheses").or(Err(Error::SqlError))?;
            let assertion: String = row.try_get("assertion").or(Err(Error::SqlError))?;
            let proof: String = row.try_get("proof").or(Err(Error::SqlError))?;

            let proof = if &proof == "" { None } else { Some(proof) };

            result.push(Theorem {
                name,
                description,
                disjoints: string_rep_to_disjoints(&disjoints_rep),
                hypotheses: string_rep_to_hypotheses(&hypotheses_rep),
                assertion,
                proof,
            });
        }
    }

    Ok(result)
}

pub async fn add_theorem(
    state: &tauri::State<'_, Mutex<AppState>>,
    name: &str,
    description: &str,
    disjoints: &Vec<String>,
    hypotheses: &Vec<Hypothesis>,
    assertion: &str,
    proof: Option<&str>,
) -> Result<(), Error> {
    let disjoints_rep = disjoints_to_string_rep(disjoints);
    let hypotheses_rep = hypotheses_to_string_rep(hypotheses);

    let mut app_state = state.lock().await;

    if let Some(ref mut conn) = app_state.db_conn {
        sqlx::query(sql::THEOREM_ADD)
            .bind(name)
            .bind(description)
            .bind(disjoints_rep)
            .bind(hypotheses_rep)
            .bind(assertion)
            .bind(proof)
            .execute(conn)
            .await
            .or(Err(Error::SqlError))?;
    }

    if let Some(ref mut mm_data) = app_state.metamath_data {
        mm_data.add_theorem(name, description, disjoints, hypotheses, assertion, proof);
    }

    Ok(())
}

fn disjoints_to_string_rep(disjoints: &Vec<String>) -> String {
    let mut disjoints_rep = String::new();

    for disj in disjoints {
        disjoints_rep.push_str(&disj);
        disjoints_rep.push('$');
    }
    disjoints_rep.pop();
    disjoints_rep
}

fn string_rep_to_disjoints(string: &str) -> Vec<String> {
    let mut disjoints = Vec::new();

    for s in string.split('$') {
        if s != "" {
            disjoints.push(s.to_string())
        }
    }
    disjoints
}

fn hypotheses_to_string_rep(hypotheses: &Vec<Hypothesis>) -> String {
    let mut hypotheses_rep = String::new();

    for hypo in hypotheses {
        hypotheses_rep.push_str(&hypo.label);
        hypotheses_rep.push('$');
        hypotheses_rep.push_str(&hypo.hypothesis);
        hypotheses_rep.push('$');
    }
    hypotheses_rep.pop();
    hypotheses_rep
}

fn string_rep_to_hypotheses(string: &str) -> Vec<Hypothesis> {
    let mut hypotheses = Vec::new();
    let mut iter = string.split('$');
    loop {
        if let Some(s1) = iter.next() {
            if let Some(s2) = iter.next() {
                hypotheses.push(Hypothesis {
                    label: s1.to_string(),
                    hypothesis: s2.to_string(),
                })
            } else {
                break;
            }
        } else {
            break;
        }
    }
    hypotheses
}

mod sql {
    pub const THEOREMS_GET: &str = "SELECT * FROM theorem;";

    pub const THEOREM_ADD: &str = "\
INSERT INTO theorem (name, description, disjoints, hypotheses, assertion, proof)
VALUES (?, ?, ?, ?, ?, ?)";
}
