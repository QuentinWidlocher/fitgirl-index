use std::error::Error;

use crate::components::release_card::ReleaseCard;

use super::get_connection;
use rusqlite::params_from_iter;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct SearchParams {
    pub title: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    pub page: Option<usize>,
}

pub fn search_db(params: SearchParams) -> Result<Vec<ReleaseCard>, Box<dyn Error>> {
    let connection = get_connection();

    let mut query = "
      SELECT r.id, r.title, r.coverSrc, r.id
      FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id"
        .to_owned();

    let mut where_clauses: Vec<&str> = vec![];
    let mut where_params: Vec<String> = vec![];

    if let Some(title) = &params.title {
        where_clauses.push("lower(r.title) LIKE ('%' || ? || '%')");
        where_params.push(title.to_lowercase());
    }

    if let Some(genre) = &params.genre {
        where_clauses.push("rg.genre = ?");
        where_params.push(genre.to_string());
    }

    let mut where_clause = "".to_owned();
    where_clauses.iter().enumerate().for_each(|(idx, clause)| {
        if idx == 0 {
            where_clause.push_str("\n\tWHERE ");
        } else {
            where_clause.push_str("\n\tand ");
        }

        where_clause.push_str(clause);
    });

    query.push_str(where_clause.as_str());
    query.push_str(
        "
      GROUP BY r.id
      ORDER BY published DESC
      LIMIT 30
    ",
    );

    if let Some(page) = params.page {
        query.push_str(format!(" OFFSET {}", (page - 1) * 30).as_str());
    }

    let mut statement = connection.prepare(query.as_str())?;

    let res = statement
        .query_map(params_from_iter(where_params), |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
            })
        })
        .map(|row| row.filter_map(|item| item.ok()).peekable())?
        .collect::<Vec<_>>();

    Ok(res)
}
