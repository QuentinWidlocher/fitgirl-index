use maud::html;
use maud::Markup;

use crate::components::release_card::release_card;

use super::release_card::ReleaseCard;

pub fn home_page(
    list: impl Iterator<Item = ReleaseCard>,
    genres: impl Iterator<Item = String>,
) -> Markup {
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
                                "Genre"
                            }
                            select name="genre" class="bg-gray-700 border border-transparent shadow-lg focus:border focus:border-red-500 focus:shadow-red-900/30 outline-none rounded px-3 py-2" {
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
