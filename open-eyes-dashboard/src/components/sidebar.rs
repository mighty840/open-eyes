use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::*;
use dioxus_free_icons::Icon;

use crate::app::Route;
use crate::components::language_switcher::LanguageSwitcher;

struct NavItem {
    label_de: &'static str,
    label_en: &'static str,
    route: Route,
    icon: Element,
}

#[component]
pub fn Sidebar(language: String, on_language_change: EventHandler<String>) -> Element {
    let current_route = use_route::<Route>();
    let mut collapsed = use_signal(|| false);

    let nav_items = [
        NavItem {
            label_de: "Startseite",
            label_en: "Home",
            route: Route::HomePage {},
            icon: rsx! { Icon { icon: BsHouseDoor, width: 18, height: 18 } },
        },
        NavItem {
            label_de: "Datenkatalog",
            label_en: "Catalog",
            route: Route::CatalogPage {},
            icon: rsx! { Icon { icon: BsCollection, width: 18, height: 18 } },
        },
        NavItem {
            label_de: "Auswertungen",
            label_en: "Analytics",
            route: Route::AnalyticsPage {},
            icon: rsx! { Icon { icon: BsBarChartLine, width: 18, height: 18 } },
        },
        NavItem {
            label_de: "Einstellungen",
            label_en: "Settings",
            route: Route::SettingsPage {},
            icon: rsx! { Icon { icon: BsGear, width: 18, height: 18 } },
        },
    ];

    let sidebar_class = if collapsed() {
        "sidebar collapsed"
    } else {
        "sidebar"
    };

    rsx! {
        nav { class: "{sidebar_class}",
            div { class: "sidebar-header",
                Icon { icon: BsEye, width: 24, height: 24 }
                if !collapsed() {
                    h1 { "Open Eyes" }
                }
            }
            div { class: "sidebar-nav",
                for item in nav_items {
                    {
                        let is_active = match (&current_route, &item.route) {
                            (Route::DatasetDetailPage { .. }, Route::CatalogPage {}) => true,
                            (a, b) => a == b,
                        };
                        let class = if is_active { "nav-item active" } else { "nav-item" };
                        let label = if language == "de" { item.label_de } else { item.label_en };
                        rsx! {
                            Link {
                                to: item.route.clone(),
                                class: class,
                                {item.icon}
                                if !collapsed() {
                                    span { "{label}" }
                                }
                            }
                        }
                    }
                }
            }
            if !collapsed() {
                div { class: "sidebar-footer",
                    LanguageSwitcher {
                        current: language,
                        on_change: move |lang: String| on_language_change(lang),
                    }
                }
            }
            button {
                class: "sidebar-toggle",
                onclick: move |_| collapsed.set(!collapsed()),
                if collapsed() {
                    Icon { icon: BsChevronRight, width: 14, height: 14 }
                } else {
                    Icon { icon: BsChevronLeft, width: 14, height: 14 }
                }
            }
        }
    }
}
