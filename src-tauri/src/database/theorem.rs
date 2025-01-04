use super::Error;
use crate::{
    metamath,
    model::{Hypothesis, MetamathData, TheoremPath},
};
use sqlx::SqliteConnection;

pub fn calc_db_index_for_theorem(
    metamath_data: &MetamathData,
    insert_path: &TheoremPath,
) -> Result<i32, metamath::Error> {
    let mut sum = -1; // Start at -1 to not count the top-most header, which is not stored in the db

    let mut header = &metamath_data.theorem_list_header;

    for &pos_index in &insert_path.header_path.path {
        sum += 1;
        sum += header.theorems.len() as i32;
        for index in 0..pos_index {
            sum += header
                .sub_headers
                .get(index)
                .ok_or(metamath::Error::InternalLogicError)?
                .count_theorems_and_headers();
        }
        header = header
            .sub_headers
            .get(pos_index)
            .ok_or(metamath::Error::InternalLogicError)?;
    }

    if header.theorems.len() >= insert_path.theorem_index {
        sum += 1;
        sum += insert_path.theorem_index as i32;

        Ok(sum)
    } else {
        Err(metamath::Error::InternalLogicError)
    }
}

pub async fn add_theorem_database(
    conn: &mut SqliteConnection,
    db_index: i32,
    name: &str,
    description: &str,
    disjoints: &Vec<String>,
    hypotheses: &Vec<Hypothesis>,
    assertion: &str,
    proof: Option<&str>,
) -> Result<(), Error> {
    let disjoints_rep = disjoints_to_string_rep(disjoints);
    let hypotheses_rep = hypotheses_to_string_rep(hypotheses);

    sqlx::query(sql::THEOREM_ADD)
        .bind(db_index)
        .bind(name)
        .bind(description)
        .bind(disjoints_rep)
        .bind(hypotheses_rep)
        .bind(assertion)
        .bind(proof)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

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

mod sql {
    pub const THEOREM_ADD: &str = "\
UPDATE theorem
SET db_index = db_index + 1
WHERE db_index >= $1;

UPDATE header
SET db_index = db_index + 1
WHERE db_index >= $1;

INSERT INTO theorem (db_index, name, description, disjoints, hypotheses, assertion, proof)
VALUES ($1, $2, $3, $4, $5, $6, $7);";
}
