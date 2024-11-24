use std::error::Error;

use chrono::NaiveDateTime;

use super::{get_client, Release, ReleaseWithInfo};

pub async fn get_release(id: String) -> Result<ReleaseWithInfo, Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let row = client
		.query_one(
			"  SELECT title,
       coverSrc,
       link,
       published,
       originalSize,
       repackSize,
       mirrors,
       screenshots,
       repackDescription,
       gameDescription
       FROM
              releases
       WHERE id = $1
        ",
			&[&id],
		)
		.await?;

	let release = Release {
		title: row.get(0),
		cover_src: row.get(1),
		link: row.get(2),
		published: row.get::<usize, NaiveDateTime>(3),
		original_size: row.get(4),
		repack_size: row.get(5),
		mirrors: serde_json::from_str(row.get::<usize, String>(6).as_str()).unwrap_or(vec![]),
		screenshots: serde_json::from_str(row.get::<usize, String>(7).as_str()).unwrap_or(vec![]),
		repack_description: row.get(8),
		game_description: row.get(9),
	};

	let genres = client.query("SELECT rg.genre FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id WHERE r.id = $1 ", &[&id]).await?
    .iter()
    .map(|row| row.get(0))
		.collect::<Vec<String>>();

	Ok(ReleaseWithInfo { release, genres })
}
