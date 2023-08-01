use axum::extract::Path;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query;
use components::generic_error::generic_error;
use components::home_page::home_page;
use db::genres::genres;
use db::last_releases::last_releases;
use db::search::search_db;
use db::search::SearchParams;

use maud::html;
use services::htmx_boosting::htmx_boosting;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use uuid::Uuid;

use crate::components::release_card::release_card;
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
    let query_params_next_page = SearchParams {
        title: query_params.title.clone(),
        genre: query_params.genre.clone(),
        page: Some(query_params.page.unwrap_or(1) + 1),
    };

    let query_params_next_page =
        serde_html_form::to_string(&query_params_next_page).unwrap_or("".to_string());

    let list = search_db(query_params);

    if let Ok(list) = list {
        if list.is_empty() {
            html! {}
        } else {
            html! {
              @for item in list {
                (release_card(&item))
              }
              span hx-swap="outerHTML" hx-trigger="revealed" hx-get={ ( format!( "/search?{}", query_params_next_page ) ) } {}
            }
        }
    } else {
        html! {}
    }
}

async fn index() -> impl IntoResponse {
    let list = last_releases();
    let genres = genres();

    let list = match list {
        Ok(list) => list,
        Err(err) => return generic_error(err.to_string()),
    };

    let genres = match genres {
        Ok(genres) => genres,
        Err(err) => return generic_error(err.to_string()),
    };

    home_page(list.into_iter(), genres.into_iter())
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

// fn prout(state: AppState) -> Result<String, Box<dyn Error>> {
//     let connection = get_connection_safe(&state.db_path)?;
//
//     let mut releases_stmt = connection
//         .prepare("SELECT id, title, coverSrc FROM releases ORDER BY published DESC LIMIT 20")?;
//
//     let result = releases_stmt
//         .query_map([], |row| row.get::<usize, String>(0))?
//         .filter_map(|x| x.ok())
//         .collect::<Vec<String>>()
//         .join("\n");
//
//     Ok(result)
//
//     // let files: Result<Vec<_>, io::Error> =
//     //     fs::read_dir("opt/shuttle/shuttle-builds/fitgirl-index/db")?
//     //         .map(|res| res.map(|e| e.path()))
//     //         .collect();
//     //
//     // Ok(files?
//     //     .into_iter()
//     //     .map(|x| x.to_str().unwrap_or("").to_string())
//     //     .collect::<Vec<String>>()
//     //     .join("\n"))
// }

// async fn test(State(state): State<AppState>) -> impl IntoResponse {
//     match prout(state) {
//         Ok(result) => result,
//         Err(e) => e.to_string(),
//     }
// }

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    #[cfg(debug_assertions)]
    {
        let app = Router::new()
            .nest_service("/assets", ServeDir::new(ASSETS_PATH))
            .route("/", get(index))
            .route("/release/:id", get(release))
            .layer(middleware::from_fn(htmx_boosting))
            .layer(LiveReloadLayer::new())
            .route("/search", get(search));

        Ok(app.into())
    }

    #[cfg(not(debug_assertions))]
    {
        let app = Router::new()
            .nest_service("/assets", ServeDir::new(ASSETS_PATH))
            .route("/", get(index))
            .route("/release/:id", get(release))
            .layer(middleware::from_fn(htmx_boosting))
            .route("/search", get(search));
        Ok(app.into())
    }
}
