use dioxus::prelude::*;

static CHART_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[component]
pub fn ChartContainer(option_json: String) -> Element {
    let chart_id = use_signal(|| {
        let n = CHART_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("echart-{n}")
    });

    let id = chart_id();
    let json = option_json.clone();

    use_effect(move || {
        let id = id.clone();
        let json = json.clone();
        if json.is_empty() || json == "{}" {
            return;
        }
        spawn(async move {
            let js = format!(
                r#"
                (function() {{
                    var el = document.getElementById('{id}');
                    if (!el || !window.echarts) return;
                    var existing = echarts.getInstanceByDom(el);
                    if (existing) existing.dispose();
                    var chart = echarts.init(el, 'dark');
                    chart.setOption({json});
                    window.addEventListener('resize', function() {{ chart.resize(); }});
                }})();
                "#
            );
            let _ = document::eval(&js);
        });
    });

    rsx! {
        div {
            id: "{chart_id}",
            class: "chart-container",
        }
    }
}
