use crate::{model::HtmlRepresentation, Error};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

pub async fn get_html_representations_database(
    conn: &mut SqliteConnection,
) -> Result<Vec<HtmlRepresentation>, Error> {
    let mut html_representations = Vec::new();
    let mut rows = sqlx::query(sql::HTML_REPRESENTATIONS_GET).fetch(conn);

    while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
        let symbol = row.try_get("symbol").or(Err(Error::SqlError))?;
        let html = row.try_get("html").or(Err(Error::SqlError))?;

        html_representations.push(HtmlRepresentation { symbol, html });
    }

    Ok(html_representations)
}

pub async fn set_html_representations_database(
    conn: &mut SqliteConnection,
    html_representations: &Vec<HtmlRepresentation>,
) -> Result<(), Error> {
    sqlx::query(sql::HTML_REPRESENTATIONS_DELETE)
        .execute(&mut *conn)
        .await
        .or(Err(Error::SqlError))?;

    for (index, html_representation) in html_representations.iter().enumerate() {
        sqlx::query(sql::HTML_REPRESENTATION_ADD)
            .bind(index as i32)
            .bind(&html_representation.symbol)
            .bind(&html_representation.html)
            .execute(&mut *conn)
            .await
            .or(Err(Error::SqlError))?;
    }

    Ok(())
}

mod sql {
    pub const HTML_REPRESENTATIONS_GET: &str = "SELECT * FROM html_representation;";

    pub const HTML_REPRESENTATIONS_DELETE: &str = "DELETE FROM html_representation;";

    pub const HTML_REPRESENTATION_ADD: &str = "\
INSERT INTO html_representation ([index], symbol, html)
VALUES ($1, $2, $3);";
}
