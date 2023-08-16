use std::error::Error;

use chrono::NaiveDateTime;
use rusqlite::Connection;

use crate::components::release_card::ReleaseCard;

use super::get_connection;

pub fn last_releases() -> Result<Vec<ReleaseCard>, Box<dyn Error>> {
    let connection = get_connection();

    let mut releases_stmt = connection.prepare(
        "SELECT id, title, coverSrc, published FROM releases ORDER BY published DESC LIMIT 30",
    )?;

    let result = releases_stmt
        .query_map([], |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
                published: NaiveDateTime::parse_from_str(
                    row.get::<usize, String>(3)?.as_str(),
                    "%Y-%m-%dT%H:%M:%S.000Z",
                )
                .unwrap(),
            })
        })?
        .filter_map(|row| row.ok())
        .collect::<Vec<ReleaseCard>>();

    Ok(result)
}

pub fn last_release(connection: &Connection) -> Result<ReleaseCard, Box<dyn Error>> {
    let mut releases_stmt = connection.prepare(
        "SELECT id, title, coverSrc, published FROM releases ORDER BY published DESC LIMIT 1",
    )?;

    let result = releases_stmt
        .query_map([], |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
                published: NaiveDateTime::parse_from_str(
                    row.get::<usize, String>(3)?.as_str(),
                    "%Y-%m-%dT%H:%M:%S.000Z",
                )
                .unwrap(),
            })
        })?
        .filter_map(|row| row.ok())
        .last()
        .ok_or("No last release")?;

    Ok(result)
}
