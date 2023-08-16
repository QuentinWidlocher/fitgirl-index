use maud::html;
use maud::Markup;
use maud::PreEscaped;

use crate::db::Release;

pub fn release_page(release: Release) -> Markup {
    let mut title = release.title.splitn(2, ['-', '–']);

    html! {
        main class="container mt-10 pt-20 mx-auto grid grid-cols-1 grid-rows-[auto_1fr] lg:grid-rows-1 lg:grid-cols-[1fr_2fr] gap-5" {
            section class="flex flex-col gap-10" {
                img class="release-cover lg:rounded-xl shadow-xl" src={(release.cover_src)} {}
                ul class="grid grid-cols-3 gap-2" {
                    @for (index, screenshot) in release.screenshots.into_iter().skip(1).enumerate().collect::<Vec<(usize, String)>>() {
                        li id={(format!("screenshot_{}", index))} {
                            img hx-get="/fullscreen-screenshot" hx-include={(format!("#screenshot_{} [name='original_url']", index))} hx-swap="afterend" class="rounded shadow cursor-pointer" src={(screenshot)} {}
                            input type="hidden" name="original_url" value={(screenshot)} {}
                        }
                    }
                }
            }
            section class="bg-gray-600 rounded-t-xl lg:rounded-b-xl p-10" {
                a class="hover:underline underline-offset-4" href={(release.link)} {
                    h1 class="text-5xl font-bold release-title" { (title.next().unwrap_or(release.title.as_str())) }
                    h2 class="text-xl ml-1 text-slate-400" {( title.next().unwrap_or("") )}
                }

                div class="flex gap-5 my-5" {
                    div class="text-gray-300" { span { "Published : " } strong { (release.published.format("%d/%m/%Y").to_string()) } }
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
