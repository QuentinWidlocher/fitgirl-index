use std::error::Error;

use axum::middleware;
use http::header::{ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN};

use axum::body::Body;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use components::fullscreen_screenshot::fullscreen_screenshot;
use components::generic_error::generic_error;
use components::home_page::home_page;
use components::release_list::release_list;
use db::genres::genres;
use db::last_releases::last_releases;
use db::search::search_db;
use db::search::SearchParams;
use dotenv::dotenv;

use hyper::Request;
use hyper::StatusCode;
use log::LevelFilter;
use maud::html;
use services::htmx_boosting::htmx_boosting;
use tower_http::services::ServeDir;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use services::sync_fitgirl_rss::sync_all_releases;
use services::sync_fitgirl_rss::sync_fitgirl_rss;
use simple_logger::SimpleLogger;

use tracing::error;
use uuid::Uuid;

use crate::components::release_page::release_page;
use crate::db::release::get_release;

mod components;
mod db;
mod services;

const ASSETS_PATH: &str = "src/assets";

#[axum::debug_handler]
async fn search(Query(query_params): Query<SearchParams>) -> impl IntoResponse {
	let query_params_str = serde_html_form::to_string(&query_params).unwrap_or("".to_string());

	let mut headers = HeaderMap::new();
	if let Ok(query_params_str_formatted) = format!("?{}", query_params_str).parse() {
		headers.insert("hx-push-url", query_params_str_formatted);
	}

	let result = search_db(&query_params).await;

	let html = if let Ok((list, total)) = result {
		if list.is_empty() {
			html! {}
		} else {
			let show_next_page = list.len() < total.try_into().unwrap();
			release_list(list.into_iter(), show_next_page, query_params, Some(total))
		}
	} else {
		html! {}
	};

	(headers, html)
}

#[axum::debug_handler]
async fn index(Query(query_params): Query<SearchParams>) -> impl IntoResponse {
	let query_params_str = serde_html_form::to_string(&query_params).unwrap_or("".to_string());

	let mut headers = HeaderMap::new();

	let genres = match genres().await {
		Ok(genres) => genres,
		Err(err) => return (headers, generic_error(err.to_string())),
	};

	let result = if query_params.genre.is_some() || query_params.title.is_some() {
		if let Ok(query_params_str_formatted) = format!("?{}", query_params_str).parse() {
			headers.insert("hx-push-url", query_params_str_formatted);
		}
		search_db(&query_params).await
	} else {
		last_releases().await
	};

	let (list, total) = match result {
		Ok((list, total)) => (list, total),
		Err(err) => return (headers, generic_error(err.to_string())),
	};

	let show_next_page = list.len() < total.try_into().unwrap();

	(
		headers,
		home_page(
			list.into_iter(),
			genres.into_iter(),
			show_next_page,
			query_params,
			total.into(),
		),
	)
}

#[axum::debug_handler]
async fn release(Path(id): Path<Uuid>) -> impl IntoResponse {
	let release = get_release(id.to_string()).await;

	if let Ok(release) = release {
		release_page(release)
	} else {
		generic_error("Release not found".to_string()).into()
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
		Ok(titles) => (StatusCode::OK, titles.len().to_string()),
		Err(err) => {
			println!("Error syncing db: {}", err);
			error!("Error syncing db: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
		}
	};
}

pub fn get_router() -> Result<Router, Box<dyn Error + Send + Sync>> {
	SimpleLogger::new()
		.with_utc_timestamps()
		.with_level(LevelFilter::Info)
		.init()
		.unwrap();

	let trace_layer = TraceLayer::new_for_http()
		.on_request(|_: &Request<Body>, _: &tracing::Span| tracing::info!(message = "begin request!"));

	let cors_layer = CorsLayer::new()
		.allow_headers([ACCEPT, ACCEPT_ENCODING, AUTHORIZATION, CONTENT_TYPE, ORIGIN])
		.allow_methods(tower_http::cors::Any)
		.allow_origin(tower_http::cors::Any);

	let app = Router::new()
		.nest_service("/assets", ServeDir::new(ASSETS_PATH))
		.route("/", get(index))
		.route("/release/:id", get(release))
		.layer(middleware::from_fn(htmx_boosting))
		.route("/search", get(search))
		.route("/fullscreen-screenshot", get(fullscreen_screenshot))
		.route("/db/sync", get(sync_db))
		.route("/db/sync_all", get(sync_all_db))
		.layer(cors_layer)
		.layer(trace_layer);

	Ok(app)
}

#[tokio::main]
async fn main() {
	dotenv().ok();

	let router = get_router().unwrap();
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, router).await.unwrap();
}
