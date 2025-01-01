use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use super::Error;
use crate::model::InProgressTheorem;

pub async fn get_in_progress_theorems_database(
    conn: &mut SqliteConnection,
) -> Result<Vec<InProgressTheorem>, Error> {
    let mut result = Vec::new();

    let mut rows = sqlx::query(sql::IN_PROGRESS_THEOREMS_GET).fetch(conn);

    while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
        let name: String = row.try_get("name").or(Err(Error::SqlError))?;
        let text: String = row.try_get("text").or(Err(Error::SqlError))?;

        result.push(InProgressTheorem { name, text });
    }

    Ok(result)
}

pub async fn add_in_progress_theorem_database(
    conn: &mut SqliteConnection,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    sqlx::query(sql::IN_PROGRESS_THEOREM_ADD)
        .bind(name)
        .bind(text)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
}

pub async fn set_in_progress_theorem_name_database(
    conn: &mut SqliteConnection,
    old_name: &str,
    new_name: &str,
) -> Result<(), Error> {
    sqlx::query(sql::IN_PROGRESS_THEOREM_NAME_UPDATE)
        .bind(new_name)
        .bind(old_name)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
}

pub async fn set_in_progress_theorem_text_database(
    conn: &mut SqliteConnection,
    name: &str,
    text: &str,
) -> Result<(), Error> {
    sqlx::query(sql::IN_PROGRESS_THEOREM_TEXT_UPDATE)
        .bind(text)
        .bind(name)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
}

pub async fn delete_in_progress_theorem_database(
    conn: &mut SqliteConnection,
    name: &str,
) -> Result<(), Error> {
    sqlx::query(sql::IN_PROGRESS_THEOREM_DELETE)
        .bind(name)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
}

mod sql {

    pub const IN_PROGRESS_THEOREMS_GET: &str = "SELECT * FROM in_progress_theorem;";

    pub const IN_PROGRESS_THEOREM_ADD: &str =
        "INSERT INTO in_progress_theorem (name, text) VALUES (?, ?)";

    pub const IN_PROGRESS_THEOREM_NAME_UPDATE: &str = "UPDATE in_progress_theorem
      SET name = ?
      WHERE name = ?;";

    pub const IN_PROGRESS_THEOREM_TEXT_UPDATE: &str = "UPDATE in_progress_theorem
        SET text = ?
        WHERE name = ?;";

    pub const IN_PROGRESS_THEOREM_DELETE: &str = "DELETE FROM in_progress_theorem WHERE name = ?";
}
