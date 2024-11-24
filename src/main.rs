use std::net::SocketAddr;

use axum::extract::Path;
use axum::http::HeaderMap;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query;
use components::fullscreen_screenshot::fullscreen_screenshot;
use components::generic_error::generic_error;
use components::home_page::home_page;
use components::release_list::release_list;
use db::genres::genres;
use db::last_releases::last_releases;
use db::search::search_db;
use db::search::SearchParams;

use hyper::StatusCode;
use maud::html;
use services::htmx_boosting::htmx_boosting;

use services::sync_fitgirl_rss::sync_all_releases;
use services::sync_fitgirl_rss::sync_fitgirl_rss;
use tower_http::services::ServeDir;

use tracing::error;
use uuid::Uuid;

use crate::components::release_page::release_page;
use crate::db::release::get_release;

mod components;
mod db;
mod services;

#[cfg(debug_assertions)]
const ASSETS_PATH: &str = "assets";

#[cfg(not(debug_assertions))]
const ASSETS_PATH: &str = "opt/shuttle/shuttle-builds/fitgirl-index/assets";

async fn search(Query(query_params): Query<SearchParams>) -> impl IntoResponse {
	let query_params_str = serde_html_form::to_string(&query_params).unwrap_or("".to_string());

	let mut headers = HeaderMap::new();
	if let Ok(query_params_str_formatted) = format!("?{}", query_params_str).parse() {
		headers.insert("hx-push-url", query_params_str_formatted);
	}

	let result = search_db(&query_params);

	let html = if let Ok((list, total)) = result {
		if list.is_empty() {
			html! {}
		} else {
			let show_next_page = (list.len() as i16) < total;
			release_list(list.into_iter(), show_next_page, query_params)
		}
	} else {
		html! {}
	};

	(headers, html)
}

async fn index(Query(query_params): Query<SearchParams>) -> impl IntoResponse {
	let result = if query_params.genre.is_some() || query_params.title.is_some() {
		search_db(&query_params)
	} else {
		last_releases()
	};

	let genres = genres();

	let (list, total) = match result {
		Ok((list, total)) => (list, total),
		Err(err) => return generic_error(err.to_string()),
	};

	let genres = match genres {
		Ok(genres) => genres,
		Err(err) => return generic_error(err.to_string()),
	};

	let show_next_page = (list.len() as i16) < total;

	home_page(
		list.into_iter(),
		genres.into_iter(),
		show_next_page,
		query_params,
		total,
	)
}

#[axum::debug_handler]
async fn release(Path(id): Path<Uuid>) -> impl IntoResponse {
	let release = get_release(id.to_string());

	if let Ok(release) = release {
		release_page(release)
	} else {
		generic_error("Release not found".to_string())
	}
}

#[axum::debug_handler]
async fn sync_db() -> impl IntoResponse {
	match sync_fitgirl_rss().await {
		Ok(titles) => (StatusCode::OK, titles.join("\n")),
		Err(err) => {
			println!("Error syncing db: {}", err);
			error!("Error syncing db: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
		}
	}
}

#[axum::debug_handler]
async fn sync_all_db() -> impl IntoResponse {
	match sync_all_releases().await {
		Ok(titles) => (StatusCode::OK, titles.join("\n")),
		Err(err) => {
			println!("Error syncing db: {}", err);
			error!("Error syncing db: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
		}
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
	println!("listening on {}", addr);

	#[cfg(debug_assertions)]
	{
		let app = Router::new()
			.nest_service("/assets", ServeDir::new(ASSETS_PATH))
			.route("/", get(index))
			.route("/release/:id", get(release))
			.layer(middleware::from_fn(htmx_boosting))
			.route("/search", get(search))
			.route("/fullscreen-screenshot", get(fullscreen_screenshot))
			.route("/db/sync", get(sync_db))
			.route("/db/sync_all", get(sync_all_db));

		axum::Server::bind(&addr)
			.serve(app.into_make_service())
			.await
			.unwrap();
	}

	#[cfg(not(debug_assertions))]
	{
		let app = Router::new()
			.nest_service("/assets", ServeDir::new(ASSETS_PATH))
			.route("/", get(index))
			.route("/release/:id", get(release))
			.layer(middleware::from_fn(htmx_boosting))
			.route("/search", get(search))
			.route("/fullscreen-screenshot", get(fullscreen_screenshot))
			.route("/db/sync", get(sync_db))
			.route("/db/sync_all", get(sync_all_db));

		axum::Server::bind(&addr)
			.serve(app.into_make_service())
			.await
			.unwrap();
	}

	Ok(())
}
