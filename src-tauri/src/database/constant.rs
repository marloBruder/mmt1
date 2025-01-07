use crate::{model::Constant, Error};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

pub async fn get_constants_database(conn: &mut SqliteConnection) -> Result<Vec<Constant>, Error> {
    let mut constants = Vec::new();

    let mut rows = sqlx::query(sql::CONSTANTS_GET).fetch(conn);

    while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
        let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;

        constants.push(Constant { symbol })
    }

    Ok(constants)
}

pub async fn add_constant_database(
    conn: &mut SqliteConnection,
    const_index: i32,
    symbol: &str,
) -> Result<(), Error> {
    sqlx::query(sql::CONSTANT_ADD)
        .bind(const_index)
        .bind(symbol)
        .execute(conn)
        .await
        .or(Err(Error::SqlError))?;

    Ok(())
}

pub async fn set_constants_database(
    conn: &mut SqliteConnection,
    symbols: &Vec<&str>,
) -> Result<(), Error> {
    sqlx::query(sql::CONSTANTS_DELETE)
        .execute(&mut *conn)
        .await
        .or(Err(Error::SqlError))?;

    for (index, &symbol) in symbols.iter().enumerate() {
        sqlx::query(sql::CONSTANT_ADD)
            .bind(index.to_string())
            .bind(symbol)
            .execute(&mut *conn)
            .await
            .or(Err(Error::SqlError))?;
    }

    Ok(())
}

mod sql {
    pub const CONSTANTS_GET: &str = "\
SELECT * FROM constant
ORDER BY [index];";

    pub const CONSTANTS_DELETE: &str = "DELETE FROM constant;";

    pub const CONSTANT_ADD: &str = "\
INSERT INTO constant ([index], symbol)
VALUES ($1, $2);";
}
