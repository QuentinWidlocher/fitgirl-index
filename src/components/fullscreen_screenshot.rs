use axum_extra::extract::Query;
use maud::{html, Markup};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FullscreenScreenshotParams {
	original_url: String,
}

pub async fn fullscreen_screenshot(
	Query(query_params): Query<FullscreenScreenshotParams>,
) -> Markup {
	let fullscreen_src = query_params.original_url.replace(".240p.jpg", "");

	html! {
		div id="fullscreen_screenshot" onclick="htmx.remove('#fullscreen_screenshot')" class="cursor-pointer p-20 z-20 fixed top-0 left-0 w-screen h-screen grid place-content-center bg-black/50" {
			img class="object-contain w-full h-full" src=(fullscreen_src) {}
		}
	}
}
