use std::error::Error;

use chrono::NaiveDateTime;
use rusqlite::params;

use super::{get_connection, Release, ReleaseWithInfo};

pub fn get_release(id: String) -> Result<ReleaseWithInfo, Box<dyn Error>> {
    let connection = get_connection();

    let mut stmt = connection.prepare(
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
                    WHERE id = ?
        ",
    )?;

    let release = stmt.query_row(params![id], |row| {
        Ok(Release {
            title: row.get(0)?,
            cover_src: row.get(1)?,
            link: row.get(2)?,
            published: NaiveDateTime::parse_from_str(
                row.get::<usize, String>(3)?.as_str(),
                "%Y-%m-%dT%H:%M:%S.000Z",
            )
            .unwrap(),
            original_size: row.get(4)?,
            repack_size: row.get(5)?,
            mirrors: serde_json::from_str(row.get::<usize, String>(6)?.as_str()).unwrap_or(vec![]),
            screenshots: serde_json::from_str(row.get::<usize, String>(7)?.as_str())
                .unwrap_or(vec![]),
            repack_description: row.get(8)?,
            game_description: row.get(9)?,
        })
    })?;

    let mut stmt = connection.prepare(
        "SELECT rg.genre FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id WHERE r.id = ? ",
    )?;

    let genres = stmt
        .query_map([id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(ReleaseWithInfo { release, genres })
}
