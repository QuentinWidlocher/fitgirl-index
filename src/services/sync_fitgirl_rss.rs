use chrono::DateTime;
use rss::Channel;
use scraper::{Element, Html, Selector};
use serde_json;
use std::error::Error;

use crate::db::{
    get_connection, last_releases::last_release, Link, Mirror, Release, ReleaseBuilder,
};
extern crate reqwest;

pub async fn sync_fitgirl_rss() -> Result<Vec<String>, Box<dyn Error>> {
    let connection = get_connection();

    let last = last_release(&connection)?;

    let res = reqwest::get("https://fitgirl-repacks.site/feed/")
        .await?
        .bytes()
        .await?;

    let channel = Channel::read_from(&res[..])?;

    let lang_stmt =
        &mut connection.prepare("INSERT OR IGNORE INTO languages (value) VALUES (?1)")?;

    let companies_stmt =
        &mut connection.prepare("INSERT OR IGNORE INTO companies (value) VALUES (?1)")?;

    let genres_stmt =
        &mut connection.prepare("INSERT OR IGNORE INTO genres (value) VALUES (?1)")?;

    let stmt = &mut connection.prepare(
        "INSERT INTO releases (
                id,
                title,
                link,
                published,
                coverSrc,
                originalSize,
                repackSize,
                mirrors,
                screenshots,
                repackDescription,
                gameDescription
            ) VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7,
                ?8,
                ?9,
                ?10,
                ?11
            )",
    )?;

    let lang_link_stmt = &mut connection.prepare(
        "INSERT OR IGNORE INTO release_language (release_id, language) VALUES (@releaseId, @language)"
    )?;

    let genre_link_stmt = &mut connection.prepare(
        "INSERT OR IGNORE INTO release_genre (release_id, genre) VALUES (@releaseId, @genre)",
    )?;

    let company_link_stmt = &mut connection.prepare(
        "INSERT OR IGNORE INTO release_company (release_id, company) VALUES (@releaseId, @company)",
    )?;

    println!("Last title : {}", &last.title);

    let mut titles_added: Vec<String> = vec![];
    channel
        .items()
        .iter()
        .skip(1)
        .take_while(|item| match &item.title {
            Some(title) => {
                println!("{}", title);
                title != &last.title
            }
            None => false,
        })
        .filter_map(|item| {
            get_release_from_item(item)
                .map_err(|err| {
                    println!("Error getting release from item: {}", err);
                    err
                })
                .ok()
        })
        .for_each(|res| {
            titles_added.push(res.0.title.clone());
            insert_data(
                res,
                lang_stmt,
                companies_stmt,
                genres_stmt,
                stmt,
                lang_link_stmt,
                genre_link_stmt,
                company_link_stmt,
            )
            .unwrap()
        });

    Ok(titles_added)
}

fn get_release_from_item(
    new_release: &rss::Item,
) -> Result<(Release, Vec<String>, Vec<String>, Vec<String>), Box<dyn Error>> {
    let mut release = ReleaseBuilder::default();
    let mut genres: Vec<String> = vec![];
    let mut companies: Vec<String> = vec![];
    let mut languages: Vec<String> = vec![];

    let title = new_release.title.as_ref().ok_or("No title")?;
    release.title(title);

    let link = new_release.link.as_ref().ok_or("No link")?;
    release.link(link);

    let pub_date = new_release.pub_date.as_ref().ok_or("No publication date")?;
    let pub_date = DateTime::parse_from_rfc2822(pub_date)?;
    release.published(pub_date.naive_utc());

    let content = new_release.content().ok_or("No content")?;
    let content = Html::parse_document(content);

    let img_selector = &Selector::parse("h3 + p > a > img")?;
    let cover_img = content.select(img_selector).next();

    if let Some(Some(cover_src)) = cover_img.map(|x| x.value().attr("src")) {
        release.cover_src(cover_src.to_string());
    }

    let sections_selector = &Selector::parse("h3 + *")?;
    for section in content.select(sections_selector) {
        let mut prev_section = section
            .prev_sibling_element()
            .ok_or("No prev sibling for section")?
            .text();

        if prev_section.any(|txt| txt.contains("Screenshots")) {
            let srcs_selector = &Selector::parse("a > img")?;
            let srcs = content
                .select(srcs_selector)
                .filter_map(|img| img.value().attr("src"))
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            release.screenshots(srcs);
        } else if prev_section.any(|txt| txt.contains("Repack")) {
            release.repack_description(section.inner_html().trim());
        } else if prev_section.any(|txt| txt.contains("Mirrors")) {
            let mirror_selector = &Selector::parse("li")?;

            let mirrors: Vec<Mirror> = section
                .select(mirror_selector)
                .map(|mirror| {
                    mirror
                        .select(&Selector::parse("a").unwrap())
                        .enumerate()
                        .fold(Mirror::default(), |mut acc, (index, a)| {
                            if index == 0 {
                                Mirror {
                                    links: vec![Link {
                                        name: a.text().collect::<String>(),
                                        link: a.value().attr("href").unwrap().to_string(),
                                    }],
                                }
                            } else {
                                acc.links.push(Link {
                                    name: a.text().collect::<String>(),
                                    link: a.value().attr("href").unwrap().to_string(),
                                });

                                acc
                            }
                        })
                })
                .collect();

            release.mirrors(mirrors);
        }
    }

    let p_selector = &Selector::parse("h3 + p")?;
    for el in content
        .select(p_selector)
        .next()
        .ok_or("No p after h3")?
        .text()
        .collect::<String>()
        .lines()
    {
        if let Some((category, value)) = el.split_once(": ").map(|(k, v)| (k.to_lowercase(), v)) {
            if category.contains("genre") {
                genres.extend(
                    value
                        .split([',', '/'])
                        .map(|x| x.trim().to_string())
                        .collect::<Vec<String>>(),
                );
            } else if category.contains("compan") {
                companies.extend(
                    value
                        .split([',', '/'])
                        .map(|x| x.trim().to_string())
                        .collect::<Vec<String>>(),
                );
            } else if category.contains("language") {
                languages.extend(
                    value
                        .split([',', '/'])
                        .map(|x| x.trim().to_string())
                        .collect::<Vec<String>>(),
                );
            } else if category.contains("original") {
                release.original_size(value);
            } else if category.contains("repack") {
                release.repack_size(value);
            }
        }
    }

    let description_selector = &Selector::parse(".su-spoiler-content")?;
    release.game_description(
        content
            .select(description_selector)
            .flat_map(|el| el.text())
            .collect::<String>(),
    );

    Ok((release.build()?, languages, companies, genres))
}

fn insert_data(
    (release, languages, companies, genres): (Release, Vec<String>, Vec<String>, Vec<String>),
    lang_stmt: &mut rusqlite::Statement,
    companies_stmt: &mut rusqlite::Statement,
    genres_stmt: &mut rusqlite::Statement,
    stmt: &mut rusqlite::Statement,
    lang_link_stmt: &mut rusqlite::Statement,
    genre_link_stmt: &mut rusqlite::Statement,
    company_link_stmt: &mut rusqlite::Statement,
) -> Result<(), Box<dyn Error>> {
    for lang in languages.iter() {
        lang_stmt.execute([lang])?;
    }

    for company in companies.iter() {
        companies_stmt.execute([company])?;
    }

    for genre in genres.iter() {
        genres_stmt.execute([genre])?;
    }

    let id = uuid::Uuid::new_v4().to_string();

    stmt.execute([
        &id,
        &release.title,
        &release.link,
        &release
            .published
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string(),
        &release.cover_src,
        &release.original_size,
        &release.repack_size,
        &serde_json::to_string(&release.mirrors).unwrap(),
        &serde_json::to_string(&release.screenshots).unwrap(),
        &release.repack_description,
        &release.game_description,
    ])?;

    for lang in languages.iter() {
        lang_link_stmt.execute([&id, lang])?;
    }

    for genre in genres.iter() {
        genre_link_stmt.execute([&id, genre])?;
    }

    for company in companies.iter() {
        company_link_stmt.execute([&id, company])?;
    }

    Ok(())
}
