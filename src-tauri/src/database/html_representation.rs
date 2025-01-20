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

mod sql {
    pub const HTML_REPRESENTATIONS_GET: &str = "SELECT * FROM html_representation;";
}
