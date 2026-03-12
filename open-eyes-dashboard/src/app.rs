use dioxus::prelude::*;

use crate::components::app_shell::AppShell;
use crate::pages::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppShell)]
        #[route("/")]
        HomePage {},
        #[route("/catalog")]
        CatalogPage {},
        #[route("/catalog/:dataset_id")]
        DatasetDetailPage { dataset_id: String },
        #[route("/analytics")]
        AnalyticsPage {},
        #[route("/settings")]
        SettingsPage {},
}

const FAVICON: Asset = asset!("/assets/favicon.svg");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const ECHARTS_JS: Asset = asset!("/assets/echarts.min.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { src: ECHARTS_JS }
        Router::<Route> {}
    }
}
