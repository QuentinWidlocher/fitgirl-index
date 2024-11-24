use std::error::Error;

use chrono::NaiveDateTime;
use tokio_postgres::Client;

use crate::components::release_card::ReleaseCard;

use super::get_client;

pub async fn last_releases() -> Result<(Vec<ReleaseCard>, i64), Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let result = client
		.query(
			"SELECT id, title, coverSrc, published FROM releases ORDER BY published DESC LIMIT 30",
			&[],
		)
		.await?
		.iter()
		.map(|row| ReleaseCard {
			id: row.get(0),
			title: row.get(1),
			cover_src: row.get(2),
			published: row.get::<usize, NaiveDateTime>(3),
		})
		.collect::<Vec<ReleaseCard>>();

	let count = client
		.query_one("SELECT count(*) FROM releases", &[])
		.await?
		.get(0);

	Ok((result, count))
}

pub async fn last_release(client: Client) -> Result<ReleaseCard, Box<dyn Error + Send + Sync>> {
	let row = client
		.query_one(
			"SELECT id, title, coverSrc, published FROM releases ORDER BY published DESC LIMIT 1",
			&[],
		)
		.await?;

	Ok(ReleaseCard {
		id: row.get(0),
		title: row.get(1),
		cover_src: row.get(2),
		published: row.get::<usize, NaiveDateTime>(3),
	})
}
