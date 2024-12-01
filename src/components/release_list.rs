use maud::html;
use maud::Markup;

use crate::components::release_card::release_card;
use crate::db::search::SearchParams;

use super::release_card::ReleaseCard;

pub fn release_list(
	list: impl Iterator<Item = ReleaseCard>,
	show_next_page: bool,
	query_params: SearchParams,
	total_games: Option<i64>,
) -> Markup {
	let query_params_next_page = SearchParams {
		title: query_params.title.clone(),
		genre: query_params.genre.clone(),
		page: Some(query_params.page.unwrap_or(1) + 1),
	};

	let query_params_next_page =
		serde_html_form::to_string(&query_params_next_page).unwrap_or("".to_string());

	html! {
		@if let Some(total_games) = total_games {
			span hx-swap-oob="textContent:#count" { (total_games) }
		}
		@for item in list {
			(release_card(&item))
		}
		@if show_next_page {
			span hx-swap="outerHTML" hx-trigger="revealed" hx-get={"/search?" (query_params_next_page)} {}
		}
	}
}
