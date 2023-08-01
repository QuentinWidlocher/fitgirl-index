use maud::html;
use maud::Markup;

pub struct ReleaseCard {
    pub id: String,
    pub title: String,
    pub cover_src: String,
}

pub fn release_card(item: &ReleaseCard) -> Markup {
    html! {
      li aria-name={(item.title)} class="relative group aspect-[3/4] shadow-lg hover:shadow-red-900/30" {
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
