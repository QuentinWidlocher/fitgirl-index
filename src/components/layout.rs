use maud::html;
use maud::Markup;
use maud::DOCTYPE;

pub fn layout(children: Markup) -> Markup {
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
