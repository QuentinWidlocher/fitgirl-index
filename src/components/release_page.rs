use maud::html;
use maud::Markup;
use maud::PreEscaped;

use crate::db::Release;

pub fn release_page(release: Release) -> Markup {
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
