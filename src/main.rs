use axum::extract::Path;
use axum::http::Request;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Query;
use axum_extra::response::Html;
use chrono::NaiveDateTime;
use hyper::Body;

use maud::html;
use maud::Markup;
use maud::PreEscaped;
use maud::DOCTYPE;
use rusqlite::params;
use rusqlite::params_from_iter;
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use tower_http::services::ServeDir;
use uuid::Uuid;

use tower_livereload::predicate::Predicate;
use tower_livereload::LiveReloadLayer;

struct ReleaseCard {
    id: String,
    title: String,
    cover_src: String,
}

#[derive(Deserialize)]
struct Link {
    name: String,
    link: String,
}

#[derive(Deserialize)]
struct Mirror {
    links: Vec<Link>,
}

struct Release {
    title: String,
    link: String,
    published: NaiveDateTime,
    cover_src: String,
    original_size: String,
    repack_size: String,
    mirrors: Vec<Mirror>,
    screenshots: Vec<String>,
    repack_description: String,
    game_description: String,
}

fn get_connection() -> Connection {
    Connection::open("fitgirl.db").unwrap()
}

fn layout(children: Markup) -> Markup {
    html! {
      (DOCTYPE)
        html {
          head {
            title { "FitGirl Index" }
            link rel="stylesheet" href="/assets/style.css" {}
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            style {
              r#"
                @keyframes fade-in {
                from { opacity: 0; }
                }

                @keyframes fade-out {
                to { opacity: 0; }
                }

                @keyframes slide-from-right {
                from { transform: translateX(30px); }
                }

                @keyframes slide-to-left {
                to { transform: translateX(-30px); }
                }

                ::view-transition-old(root) {
                animation: 90ms cubic-bezier(0.4, 0, 1, 1) both fade-out,
                    300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-to-left;
                }

                ::view-transition-new(root) {
                animation: 210ms cubic-bezier(0, 0, 0.2, 1) 90ms both fade-in,
                    300ms cubic-bezier(0.4, 0, 0.2, 1) both slide-from-right;
                }

                .release-cover {
                    view-transition-name: release-cover;
                }

                .release-title {
                    view-transition-name: release-title;
                }
              "#
            }
          }
          body ."bg-gray-800 text-white" {
            (children)
              script src="https://unpkg.com/htmx.org@1.9.3" {}
          }
        }
    }
}

fn release_card(item: &ReleaseCard) -> Markup {
    html! {
      li aria-name={(item.title)} class="relative group shadow-lg hover:shadow-red-900/30" {
        a hx-boost="true" onclick="this.style.viewTransitionName = 'release-cover';" hx-swap="scroll:#html:top transition:true" href={( format!("/release/{}", item.id) )} {
          img class="h-full w-full rounded-xl object-cover" src={( item.cover_src )} {}
          div class="absolute rounded-xl top-0 grid place-content-center h-full w-full opacity-0 transition-opacity group-hover:opacity-100 text-white text-center bg-red-900/80 p-5"  {
            h2 onclick="this.style.viewTransitionName = 'release-title';"  {
              (item.title)
            }
          }
        }
      }

    }
}

#[derive(Serialize, Deserialize)]
struct Search {
    title: Option<String>,
    #[serde(default)]
    genres: Vec<String>,
    page: Option<usize>,
}

async fn search(Query(query_params): Query<Search>) -> impl IntoResponse {
    let connection = get_connection();

    let mut query = "
      SELECT r.id, r.title, r.coverSrc, r.id
      FROM releases r INNER JOIN release_genre rg ON r.id = rg.release_id"
        .to_owned();

    let mut where_clauses: Vec<&str> = vec![];
    let mut params: Vec<String> = vec![];

    if let Some(title) = &query_params.title {
        where_clauses.push("lower(r.title) LIKE ('%' || ? || '%')");
        params.push(title.to_lowercase());
    }

    if !query_params.genres.is_empty() {
        where_clauses.push("rg.genre in (?)");
        params.push(query_params.genres.join(","));
    }

    let mut where_clause = "".to_owned();
    where_clauses.iter().enumerate().for_each(|(idx, clause)| {
        if idx == 0 {
            where_clause.push_str("\n\tWHERE ");
        } else {
            where_clause.push_str("\n\tand ");
        }

        where_clause.push_str(clause);
    });

    query.push_str(where_clause.as_str());
    query.push_str(
        "
      GROUP BY r.id
      ORDER BY published DESC
      LIMIT 30
    ",
    );

    if let Some(page) = query_params.page {
        query.push_str(format!(" OFFSET {}", (page - 1) * 30).as_str());
    }

    // println!("{}", query);

    let query_params_next_page = Search {
        title: query_params.title.clone(),
        genres: query_params.genres.clone(),
        page: Some(query_params.page.unwrap_or(1) + 1),
    };

    let query_params_next_page = serde_html_form::to_string(&query_params_next_page).unwrap();

    let mut statement = connection.prepare(query.as_str()).unwrap();

    let list = statement
        .query_map(params_from_iter(params), |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
            })
        })
        .map(|row| row.filter_map(|item| item.ok()).peekable());

    if let Ok(mut list) = list {
        if list.peek().is_none() {
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
        println!("Error: {:?}", list.err().unwrap());
        html! {
          "Empty"
        }
    }
}

async fn index() -> impl IntoResponse {
    let connection = get_connection();

    let mut releases_stmt = connection
        .prepare("SELECT id, title, coverSrc FROM releases ORDER BY published DESC LIMIT 20")
        .unwrap();

    let mut genres_stmt = connection
        .prepare("SELECT value FROM genres ORDER BY value ASC")
        .unwrap();

    let list = releases_stmt
        .query_map([], |row| {
            Ok(ReleaseCard {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_src: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|row| row.ok());

    let genres = genres_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .filter_map(|row| row.ok());

    html! {
        main class="container mx-auto" {
            h1 class="text-4xl my-5 pt-10 text-center" {
                "FitGirl Repacks Index"
            }

            div class="mx-5 grid grid-cols-1 grid-rows-[auto_1fr] h-full lg:grid-rows-1 lg:grid-cols-[1fr_3fr] gap-10" {
                form hx-get="/search" hx-target="#result" {
                    div class="flex flex-col gap-5" {
                        label class="flex flex-col" {
                            span {
                                "Search by title"
                            }
                            input name="title" class="bg-gray-700 shadow-lg border border-transparent focus:border focus:border-red-500 focus:shadow-red-900/30 outline-none h-10 rounded px-3 py-2" {}
                        }
                        label class="flex flex-col" {
                            span {
                                "Genres"
                            }
                            select multiple name="genres" class="h-52 bg-gray-700 border border-transparent shadow-lg focus:border focus:border-red-500 focus:shadow-red-900/30 outline-none rounded px-3 py-2" {
                                @for genre in genres {
                                    option value={(genre)} {
                                        (genre)
                                    }
                                }
                            }
                        }
                    }

                    div class="flex gap-2 w-full mt-10" {
                        button class="flex-1 hover:bg-red-500/30 border hover:shadow-lg hover:shadow-red-900/30 border-red-500 mt-auto text-red-100 rounded px-3 py-2" type="reset" { "Reset" }
                        button class="flex-1 bg-red-500 hover:bg-red-600 hover:shadow-lg hover:shadow-red-900/30 mt-auto text-white rounded px-3 py-2" { "Search" }
                    }
                }

                ul class="my-5 grid grid-cols-2 lg:grid-cols-5 gap-4 justify-center" id="result" {
                    @for item in list {
                        (release_card(&item))
                    }
                    span hx-swap="outerHTML" hx-trigger="revealed" hx-get="/search?page=2" {}
                }
            }
        }
    }
}

#[axum::debug_handler]
async fn release(Path(id): Path<Uuid>) -> impl IntoResponse {
    let connection = Connection::open("fitgirl.db").unwrap();

    let mut stmt = connection
        .prepare("  SELECT title, coverSrc, link, published, originalSize, repackSize, mirrors, screenshots, repackDescription, gameDescription
                    FROM releases WHERE id = ?")
        .unwrap();

    let release = stmt
        .query_row(params![id.to_string()], |row| {
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
                mirrors: serde_json::from_str(row.get::<usize, String>(6)?.as_str())
                    .unwrap_or(vec![]),
                screenshots: serde_json::from_str(row.get::<usize, String>(7)?.as_str())
                    .unwrap_or(vec![]),
                repack_description: row.get(8)?,
                game_description: row.get(9)?,
            })
        })
        .unwrap();

    html! {
        div class="fixed top-0 left-0 m-10 z-10" {
            // a hx-boost="true" hx-swap="show:window:top transition:true" class="bg-gray-600 text-white rounded-lg hover:bg-red-500 hover:shadow-lg hover:shadow-red-900/30 border-gray-800 border shadow-lg px-10 py-5 lg:px-3 lg:py-2" href="/" { "Back" }
            button class="bg-gray-600 text-white rounded-lg hover:bg-red-500 hover:shadow-lg hover:shadow-red-900/30 border-gray-800 border shadow-lg px-10 py-5 lg:px-3 lg:py-2" onclick="window.history.back()" { "Back" }
        }

        main class="container mt-10 pt-20 mx-auto grid grid-cols-1 grid-rows-[auto_1fr] lg:grid-rows-1 lg:grid-cols-[1fr_2fr] gap-5" {
            section class="flex flex-col gap-10" {
                img class="release-cover lg:rounded-xl shadow-xl" src={(release.cover_src)} {}
                ul class="grid grid-cols-3 gap-2" {
                    @for screenshot in release.screenshots.into_iter().skip(1).collect::<Vec<String>>() {
                        li {
                            img class="rounded shadow" src={(screenshot)} {}
                        }
                    }
                }
            }
            section class="bg-gray-600 rounded-t-xl lg:rounded-b-xl p-10" {
                a class="hover:underline underline-offset-4" href={(release.link)} {
                    h1 class="text-5xl font-bold release-title" { (release.title) }
                }

                div class="flex gap-5 my-5" {
                    div class="text-gray-300" { span { "Published : " } strong { (release.published.to_string()) } }
                    div class="text-gray-300" { span { "Original Size : " } strong { (release.original_size) } }
                    div class="text-gray-300" { span { "Repack Size : " } strong { (release.repack_size) } }
                }
                ul class="flex flex-col gap-5 my-10" {
                    @for mirror in release.mirrors {
                        li {
                            ul class="flex gap-1" {
                                @for link in mirror.links {
                                    li {
                                        a class="whitespace-nowrap bg-gray-500 hover:bg-red-500 hover:shadow-lg hover:shadow-red-500/30 rounded-full px-3 py-2" href={(link.link)} { (link.name) }
                                    }
                                }
                            }
                        }
                    }
                }
                p class="mt-5" { (PreEscaped(release.repack_description)) }
                p class="mt-5" { (PreEscaped(release.game_description)) }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct HtmxBoostingPredicate;

impl Predicate<Request<Body>> for HtmxBoostingPredicate {
    fn check(&mut self, request: &Request<Body>) -> bool {
        request
            .headers()
            .keys()
            .all(|header| header != "Hx-Boosted")
    }
}

async fn htmx_boosting<B>(request: Request<B>, next: Next<B>) -> Response {
    let boosted = request
        .headers()
        .keys()
        .any(|header| header == "Hx-Boosted");

    let html = request.headers().iter().any(|(header_name, header_value)| {
        header_name == "accept" && header_value.to_str().unwrap_or("").contains("text/html")
    });

    if !html || boosted {
        next.run(request).await
    } else {
        // wrap response in layout()
        let response = next.run(request).await;
        let body = response.into_body();
        let body = hyper::body::to_bytes(body).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        let body = layout(html! {
            (PreEscaped(body))
        });
        Html(body).into_response()
    }
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(index))
        .route("/release/:id", get(release))
        .layer(middleware::from_fn(htmx_boosting))
        .route("/search", get(search));

    Ok(app.into())
}
