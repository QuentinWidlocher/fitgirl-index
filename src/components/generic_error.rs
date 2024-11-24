use maud::{html, Markup};

pub fn generic_error(error: String) -> Markup {
	html! {
		h1 class="text-4xl my-5 pt-10 text-center" {
			"An error occurred"
		}
		details {
			summary { "Show details" }

			(error)
		}
	}
}
