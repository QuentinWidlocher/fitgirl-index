use chrono::DateTime;
use rss::Channel;
use scraper::{Element, ElementRef, Html, Selector};
use serde_json;
use std::{error::Error, vec};

use crate::db::{get_client, last_releases::last_release, Link, Mirror, Release, ReleaseBuilder};
extern crate reqwest;

async fn get_release_list(
	page: usize,
) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
	let res = reqwest::get(&format!(
		"https://fitgirl-repacks.site/all-my-repacks-a-z/?lcp_page0={}",
		page
	))
	.await?
	.text()
	.await?;

	let doc = Html::parse_document(&res);

	let selector = &Selector::parse(".lcp_catlist li a").unwrap();
	Ok(
		doc
			.select(selector)
			.map(|el| {
				(
					el.text().collect(),
					el.value().attr("href").unwrap().to_string(),
				)
			})
			.collect(),
	)
}

pub async fn sync_all_releases() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let mut full_game_list: Vec<(String, String)> = vec![];
	let mut page = 1;
	loop {
		println!("getting page {}", page);

		let list = get_release_list(page).await?;

		if list.is_empty() {
			break;
		}

		page += 1;

		full_game_list.extend(list);
	}

	println!("{} games on fitgirl site", full_game_list.len());

	let mut insert_list = vec![];

	let existing_titles: Vec<String> = client
		.query("SELECT title FROM releases", &[])
		.await?
		.iter()
		.map(|row| row.get::<usize, String>(0))
		.collect::<Vec<String>>();

	println!("{} games in db", existing_titles.len());

	for (title, url) in full_game_list.iter().filter(|(title, _)| {
		!existing_titles
			.iter()
			.any(|existing_title| title == existing_title)
	}) {
		println!("getting {}", title);
		let html = reqwest::get(url).await?.text().await?;

		if let Ok(data) = get_release_from_html(html) {
			let title = data.0.title.clone();
			insert_data(data).await?;
			insert_list.push(title);
		}
	}

	Ok(insert_list)
}

pub async fn sync_fitgirl_rss() -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
	let res = reqwest::get("https://fitgirl-repacks.site/feed/")
		.await?
		.bytes()
		.await?;

	let client = get_client().await;

	let last = last_release(client).await?;

	let channel = Channel::read_from(&res[..])?;

	println!("Last title : {}", &last.title);

	let mut titles_added: Vec<String> = vec![];
	for res in channel
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
	{
		titles_added.push(res.0.title.clone());
		insert_data(res).await?;
	}

	Ok(titles_added)
}

fn parse_info_from_html(
	content: ElementRef,
	release: &mut ReleaseBuilder,
) -> Result<(Release, Vec<String>, Vec<String>, Vec<String>), Box<dyn Error + Send + Sync>> {
	let mut genres: Vec<String> = vec![];
	let mut companies: Vec<String> = vec![];
	let mut languages: Vec<String> = vec![];

	let img_selector = &Selector::parse("h3 + p > a > img").map_err(|_| "bad selector".to_string())?;
	let cover_img = content.select(img_selector).next();

	if let Some(Some(cover_src)) = cover_img.map(|x| x.value().attr("src")) {
		release.cover_src(cover_src.to_string());
	}

	let sections_selector = &Selector::parse("h3 + *").map_err(|_| "bad selector".to_string())?;
	for section in content.select(sections_selector) {
		let mut prev_section = section
			.prev_sibling_element()
			.ok_or("No prev sibling for section")?
			.text();

		if prev_section.any(|txt| txt.contains("Screenshots")) {
			let srcs_selector = &Selector::parse("a > img").map_err(|_| "bad selector".to_string())?;
			let srcs = content
				.select(srcs_selector)
				.filter_map(|img| img.value().attr("src"))
				.map(|x| x.to_string())
				.collect::<Vec<String>>();

			release.screenshots(srcs);
		} else if prev_section.any(|txt| txt.contains("Repack")) {
			release.repack_description(section.inner_html().trim());
		} else if prev_section.any(|txt| txt.contains("Mirrors")) {
			let mirror_selector = &Selector::parse("li").map_err(|_| "bad selector".to_string())?;

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
										link: a.value().attr("href").unwrap_or("").to_string(),
									}],
								}
							} else {
								acc.links.push(Link {
									name: a.text().collect::<String>(),
									link: a.value().attr("href").unwrap_or("").to_string(),
								});

								acc
							}
						})
				})
				.collect();

			release.mirrors(mirrors);
		}
	}

	let p_selector = &Selector::parse("h3 + p").map_err(|_| "bad selector".to_string())?;
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

	let description_selector = &Selector::parse("h3+ul+.su-spoiler .su-spoiler-content")
		.map_err(|_| "bad selector".to_string())?;
	release.game_description(
		content
			.select(description_selector)
			.next()
			.and_then(|el| Some(el.inner_html()))
			.unwrap_or_default(),
	);

	Ok((release.build()?, languages, companies, genres))
}

fn get_release_from_html(
	html: String,
) -> Result<(Release, Vec<String>, Vec<String>, Vec<String>), Box<dyn Error + Send + Sync>> {
	let mut release = ReleaseBuilder::default();

	let html = Html::parse_document(html.as_str());

	let content_selector =
		&Selector::parse(".entry-content").map_err(|_| "bad selector".to_string())?;
	let content = html.select(content_selector).next();

	let content = content.ok_or("No Content")?;

	let pubdate_selector = &Selector::parse("meta[property='article:published_time']")
		.map_err(|_| "bad selector".to_string())?;
	let pub_date = html
		.select(pubdate_selector)
		.next()
		.ok_or("No pub date metadata")?;
	release.published(
		DateTime::parse_from_rfc3339(pub_date.value().attr("content").ok_or("Bad date format")?)?
			.naive_utc(),
	);

	let title_selector = &Selector::parse(".entry-title").map_err(|_| "bad selector".to_string())?;
	let title: String = html
		.select(title_selector)
		.next()
		.ok_or("No title")?
		.text()
		.collect();
	release.title(title);

	let link_selector =
		&Selector::parse("link[rel='canonical']").map_err(|_| "bad selector".to_string())?;
	let link = html
		.select(link_selector)
		.next()
		.ok_or("No link tag")?
		.value()
		.attr("href")
		.ok_or("No link href")?;
	release.link(link);

	parse_info_from_html(content, &mut release)
}

fn get_release_from_item(
	new_release: &rss::Item,
) -> Result<(Release, Vec<String>, Vec<String>, Vec<String>), Box<dyn Error + Send + Sync>> {
	let mut release = ReleaseBuilder::default();

	let title = new_release.title.as_ref().ok_or("No title")?;
	release.title(title);
	let link = new_release.link.as_ref().ok_or("No link")?;
	release.link(link);

	let pub_date = new_release.pub_date.as_ref().ok_or("No publication date")?;
	let pub_date = DateTime::parse_from_rfc2822(pub_date)?;
	release.published(pub_date.naive_utc());

	let content = new_release.content().ok_or("No content")?;
	let content = Html::parse_document(content);

	parse_info_from_html(content.root_element(), &mut release)
}

async fn insert_data(
	(release, languages, companies, genres): (Release, Vec<String>, Vec<String>, Vec<String>),
) -> Result<(), Box<dyn Error + Send + Sync>> {
	let client = get_client().await;

	let lang_stmt = client
		.prepare("INSERT INTO languages (value) VALUES ($1) ON CONFLICT DO NOTHING")
		.await?;

	let companies_stmt = client
		.prepare("INSERT INTO companies (value) VALUES ($1) ON CONFLICT DO NOTHING")
		.await?;

	let genres_stmt = client
		.prepare("INSERT INTO genres (value) VALUES ($1) ON CONFLICT DO NOTHING")
		.await?;

	let stmt = client
		.prepare(
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
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11
            )",
		)
		.await?;

	let lang_link_stmt = client
		.prepare(
			"INSERT INTO release_language (release_id, language) VALUES ($1, $2) ON CONFLICT DO NOTHING",
		)
		.await?;

	let genre_link_stmt = client
		.prepare("INSERT INTO release_genre (release_id, genre) VALUES ($1, $2) ON CONFLICT DO NOTHING")
		.await?;

	let company_link_stmt = client
		.prepare(
			"INSERT INTO release_company (release_id, company) VALUES ($1, $2) ON CONFLICT DO NOTHING",
		)
		.await?;

	for lang in languages.iter() {
		client.execute(&lang_stmt, &[lang]).await?;
	}

	for company in companies.iter() {
		client.execute(&companies_stmt, &[company]).await?;
	}

	for genre in genres.iter() {
		client.execute(&genres_stmt, &[genre]).await?;
	}

	let id = uuid::Uuid::new_v4().to_string();

	client
		.execute(
			&stmt,
			&[
				&id,
				&release.title,
				&release.link,
				&release.published,
				&release.cover_src,
				&release.original_size,
				&release.repack_size,
				&serde_json::to_string(&release.mirrors).unwrap(),
				&serde_json::to_string(&release.screenshots).unwrap(),
				&release.repack_description,
				&release.game_description,
			],
		)
		.await?;

	println!("inserted {}", &release.title);

	for lang in languages.iter() {
		client.execute(&lang_link_stmt, &[&id, lang]).await?;
	}

	for genre in genres.iter() {
		client.execute(&genre_link_stmt, &[&id, genre]).await?;
	}

	for company in companies.iter() {
		client.execute(&company_link_stmt, &[&id, company]).await?;
	}

	Ok(())
}
