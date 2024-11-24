use std::error::Error;

use super::get_client;

pub async fn genres() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let rows = client
		.query("SELECT value FROM genres ORDER BY value ASC", &[])
		.await?;

	let result = rows.iter().map(|row| row.get(0)).collect::<Vec<String>>();

	Ok(result)
}
