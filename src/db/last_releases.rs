use std::error::Error;

use crate::components::release_card::ReleaseCard;

use super::get_connection;

pub fn last_releases() -> Result<Vec<ReleaseCard>, Box<dyn Error>> {
    let connection = get_connection();

    let mut releases_stmt = connection
        .prepare("SELECT id, title, coverSrc FROM releases ORDER BY published DESC LIMIT 20")
        .unwrap();

    let result = releases_stmt
        .query_map([], |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect::<Vec<ReleaseCard>>();

    Ok(result)
}
