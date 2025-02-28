use crate::{
    model::{Header, HeaderPath, Hypothesis, MetamathData, Theorem},
    Error,
};
use futures::TryStreamExt;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

// pub async fn get_theorem_list_header_database(
//     conn: &mut SqliteConnection,
// ) -> Result<Header, Error> {
//     let mut main_header = Header {
//         title: "".to_string(),
//         theorems: Vec::new(),
//         sub_headers: Vec::new(),
//     };

//     let mut last_added_header = &mut main_header;

//     let db_headers = get_db_headers(conn).await?;

//     let mut rows = sqlx::query(sql::THEOREMS_GET).fetch(conn);

//     let mut next_theorem_and_index =
//         get_next_theorem(rows.try_next().await.or(Err(Error::SqlError))?).await?;

//     for db_header in db_headers {
//         while let Some((next_theorem_db_index, ref next_theorem)) = next_theorem_and_index {
//             if next_theorem_db_index < db_header.db_index {
//                 // Performace optimization might be possible here
//                 // *next_theorem should work, but the compiler won't allow it
//                 last_added_header.theorems.push(next_theorem.clone());
//                 next_theorem_and_index =
//                     get_next_theorem(rows.try_next().await.or(Err(Error::SqlError))?).await?;
//             } else {
//                 break;
//             }
//         }

//         if db_header.depth < 0 {
//             return Err(Error::InvalidDatabaseDataError);
//         } else {
//             let mut header_placement = &mut main_header;
//             for _ in 0..db_header.depth {
//                 header_placement = header_placement
//                     .sub_headers
//                     .last_mut()
//                     .ok_or(Error::InvalidDatabaseDataError)?;
//             }
//             header_placement.sub_headers.push(Header {
//                 title: db_header.title,
//                 theorems: Vec::new(),
//                 sub_headers: Vec::new(),
//             });
//             last_added_header = header_placement.sub_headers.last_mut().unwrap();
//         }
//     }

//     while let Some((_, ref next_theorem)) = next_theorem_and_index {
//         // Performace optimization might be possible here
//         // *next_theorem should work, but the compiler won't allow it
//         last_added_header.theorems.push(next_theorem.clone());
//         next_theorem_and_index =
//             get_next_theorem(rows.try_next().await.or(Err(Error::SqlError))?).await?;
//     }

//     Ok(main_header)
// }

// async fn get_next_theorem(next_row: Option<SqliteRow>) -> Result<Option<(i32, Theorem)>, Error> {
//     if let Some(row) = next_row {
//         let db_index: i32 = row.try_get("db_index").or(Err(Error::SqlError))?;
//         let name: String = row.try_get("name").or(Err(Error::SqlError))?;
//         let description: String = row.try_get("description").or(Err(Error::SqlError))?;
//         let disjoints_rep: String = row.try_get("disjoints").or(Err(Error::SqlError))?;
//         let hypotheses_rep: String = row.try_get("hypotheses").or(Err(Error::SqlError))?;
//         let assertion: String = row.try_get("assertion").or(Err(Error::SqlError))?;
//         let proof: Option<String> = row.try_get("proof").or(Err(Error::SqlError))?;

//         return Ok(Some((
//             db_index,
//             Theorem {
//                 name,
//                 description,
//                 disjoints: string_rep_to_disjoints(&disjoints_rep),
//                 hypotheses: string_rep_to_hypotheses(&hypotheses_rep),
//                 assertion,
//                 proof,
//             },
//         )));
//     }
//     Ok(None)
// }

// fn string_rep_to_disjoints(string: &str) -> Vec<String> {
//     let mut disjoints = Vec::new();

//     for s in string.split('$') {
//         if s != "" {
//             disjoints.push(s.to_string())
//         }
//     }
//     disjoints
// }

// fn string_rep_to_hypotheses(string: &str) -> Vec<Hypothesis> {
//     let mut hypotheses = Vec::new();
//     let mut iter = string.split('$');
//     loop {
//         if let Some(s1) = iter.next() {
//             if let Some(s2) = iter.next() {
//                 hypotheses.push(Hypothesis {
//                     label: s1.to_string(),
//                     hypothesis: s2.to_string(),
//                 })
//             } else {
//                 break;
//             }
//         } else {
//             break;
//         }
//     }
//     hypotheses
// }

// struct DbHeader {
//     db_index: i32,
//     depth: i32,
//     title: String,
// }

// async fn get_db_headers(conn: &mut SqliteConnection) -> Result<Vec<DbHeader>, Error> {
//     let mut result = Vec::new();

//     let mut rows = sqlx::query(sql::DB_HEADERS_GET).fetch(conn);

//     while let Some(row) = rows.try_next().await.or(Err(Error::SqlError))? {
//         let db_index: i32 = row.try_get("db_index").or(Err(Error::SqlError))?;
//         let depth: i32 = row.try_get("depth").or(Err(Error::SqlError))?;
//         let title: String = row.try_get("title").or(Err(Error::SqlError))?;

//         result.push(DbHeader {
//             db_index,
//             depth,
//             title,
//         });
//     }

//     Ok(result)
// }

// pub fn calc_db_index_for_header(
//     metamath_data: &MetamathData,
//     insert_path: &HeaderPath,
// ) -> Result<i32, Error> {
//     let mut sum = -1;

//     let mut header = &metamath_data.theorem_list_header;

//     for &pos_index in &insert_path.path {
//         sum += 1;
//         sum += header.theorems.len() as i32;
//         for index in 0..pos_index {
//             sum += header
//                 .sub_headers
//                 .get(index)
//                 .ok_or(Error::InternalLogicError)?
//                 .count_theorems_and_headers();
//         }
//         header = header
//             .sub_headers
//             .get(pos_index)
//             .ok_or(Error::InternalLogicError)?;
//     }

//     Ok(sum)
// }

// pub async fn add_header_database(
//     conn: &mut SqliteConnection,
//     db_index: i32,
//     depth: i32,
//     title: &str,
// ) -> Result<(), Error> {
//     sqlx::query(sql::HEADER_ADD)
//         .bind(db_index)
//         .bind(depth)
//         .bind(title)
//         .execute(conn)
//         .await
//         .or(Err(Error::SqlError))?;

//     Ok(())
// }

// mod sql {
//     pub const THEOREMS_GET: &str = "SELECT * FROM theorem ORDER BY db_index;";

//     pub const DB_HEADERS_GET: &str = "SELECT * FROM header ORDER BY db_index;";

//     pub const HEADER_ADD: &str = "\
// UPDATE theorem
// SET db_index = db_index + 1
// WHERE db_index >= $1;

// UPDATE header
// SET db_index = db_index + 1
// WHERE db_index >= $1;

// INSERT INTO header (db_index, depth, title)
// VALUES ($1, $2, $3);";
// }
