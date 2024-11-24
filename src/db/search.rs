use std::error::Error;

use crate::components::release_card::ReleaseCard;

use super::get_client;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde::Serialize;
use tokio_postgres::types::ToSql;

#[derive(Serialize, Deserialize)]
pub struct SearchParams {
	pub title: Option<String>,
	#[serde(default)]
	pub genre: Option<String>,
	pub page: Option<usize>,
}

pub async fn search_db(
	params: &SearchParams,
) -> Result<(Vec<ReleaseCard>, i64), Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let mut query = "
      SELECT r.id, r.title, r.coverSrc, r.published
      FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id"
		.to_owned();

	let mut count_query = "
      SELECT count(distinct r.id)
      FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id"
		.to_owned();

	let mut where_clauses: Vec<&str> = vec![];
	let mut where_params: Vec<&(dyn ToSql + Sync)> = vec![];
	let mut clause_nb = 1;

	let title_param = format!("r.title ILIKE ('%' || ${clause_nb} || '%')");
	let title = params
		.title
		.clone()
		.map(|x| x.to_lowercase())
		.unwrap_or("".to_owned());
	if !title.is_empty() {
		where_clauses.push(title_param.as_str());
		clause_nb += 1;
		println!("title {title}");
		where_params.push(&title);
	}

	let genre_params = format!("rg.genre = ${clause_nb}");
	let genre = params.genre.clone().unwrap_or("".to_owned());
	if !genre.is_empty() {
		where_clauses.push(genre_params.as_str());
		// clause_nb += 1;
		println!("genre {genre}");
		where_params.push(&genre);
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
	count_query.push_str(where_clause.as_str());

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

	println!("query {query}");

	let res = client
		.query(query.as_str(), where_params.as_slice())
		.await?
		.iter()
		.map(|row| ReleaseCard {
			id: row.get(0),
			title: row.get(1),
			cover_src: row.get(2),
			published: row.get::<usize, NaiveDateTime>(3),
		})
		.collect::<Vec<_>>();

	let count = client
		.query_one(count_query.as_str(), where_params.as_slice())
		.await?
		.get(0);
	println!("{count_query} {count}");

	Ok((res, count))
}
