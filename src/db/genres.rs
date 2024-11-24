use std::error::Error;

use super::get_connection;

pub fn genres() -> Result<Vec<String>, Box<dyn Error>> {
	let connection = get_connection();

	let mut genres_stmt = connection
		.prepare("SELECT value FROM genres ORDER BY value ASC")
		.unwrap();

	let result = genres_stmt
		.query_map([], |row| row.get::<_, String>(0))?
		.filter_map(|row| row.ok())
		.collect::<Vec<String>>();

	Ok(result)
}
