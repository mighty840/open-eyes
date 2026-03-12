use open_eyes_core::build_echart_option;
use open_eyes_core::models::chat::{ChartConfig, ChartType};

#[test]
fn test_bar_chart_option() {
    let config = ChartConfig {
        title: "Test Bar".to_string(),
        x_field: "category".to_string(),
        y_field: "value".to_string(),
        series_field: None,
    };
    let data = vec![
        serde_json::json!({"category": "A", "value": 10}),
        serde_json::json!({"category": "B", "value": 20}),
        serde_json::json!({"category": "C", "value": 15}),
    ];

    let option = build_echart_option(&ChartType::Bar, &config, &data);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();

    assert_eq!(parsed["title"]["text"].as_str().unwrap(), "Test Bar");
    assert_eq!(parsed["series"][0]["type"].as_str().unwrap(), "bar");
    assert_eq!(parsed["xAxis"]["data"].as_array().unwrap().len(), 3);
    assert_eq!(parsed["series"][0]["data"].as_array().unwrap().len(), 3);
}

#[test]
fn test_line_chart_option() {
    let config = ChartConfig {
        title: "Trend".to_string(),
        x_field: "year".to_string(),
        y_field: "count".to_string(),
        series_field: None,
    };
    let data = vec![
        serde_json::json!({"year": "2020", "count": 100}),
        serde_json::json!({"year": "2021", "count": 150}),
    ];

    let option = build_echart_option(&ChartType::Line, &config, &data);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();

    assert_eq!(parsed["series"][0]["type"].as_str().unwrap(), "line");
}

#[test]
fn test_pie_chart_option() {
    let config = ChartConfig {
        title: "Distribution".to_string(),
        x_field: "name".to_string(),
        y_field: "value".to_string(),
        series_field: None,
    };
    let data = vec![
        serde_json::json!({"name": "CSV", "value": 50}),
        serde_json::json!({"name": "JSON", "value": 30}),
    ];

    let option = build_echart_option(&ChartType::Pie, &config, &data);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();

    assert_eq!(parsed["series"][0]["type"].as_str().unwrap(), "pie");
    assert_eq!(parsed["series"][0]["data"].as_array().unwrap().len(), 2);
}

#[test]
fn test_scatter_chart_option() {
    let config = ChartConfig {
        title: "Scatter".to_string(),
        x_field: "x".to_string(),
        y_field: "y".to_string(),
        series_field: None,
    };
    let data = vec![
        serde_json::json!({"x": 1.0, "y": 2.0}),
        serde_json::json!({"x": 3.0, "y": 4.0}),
    ];

    let option = build_echart_option(&ChartType::Scatter, &config, &data);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();

    assert_eq!(parsed["series"][0]["type"].as_str().unwrap(), "scatter");
    let scatter_data = parsed["series"][0]["data"].as_array().unwrap();
    assert_eq!(scatter_data.len(), 2);
}

#[test]
fn test_none_chart_returns_empty() {
    let config = ChartConfig {
        title: "None".to_string(),
        x_field: "x".to_string(),
        y_field: "y".to_string(),
        series_field: None,
    };

    let option = build_echart_option(&ChartType::None, &config, &[]);
    assert_eq!(option, "{}");
}

#[test]
fn test_table_chart_returns_empty() {
    let config = ChartConfig {
        title: "Table".to_string(),
        x_field: "x".to_string(),
        y_field: "y".to_string(),
        series_field: None,
    };

    let option = build_echart_option(&ChartType::Table, &config, &[]);
    assert_eq!(option, "{}");
}

#[test]
fn test_bar_chart_with_string_numbers() {
    let config = ChartConfig {
        title: "String nums".to_string(),
        x_field: "label".to_string(),
        y_field: "amount".to_string(),
        series_field: None,
    };
    let data = vec![
        serde_json::json!({"label": "A", "amount": "42.5"}),
        serde_json::json!({"label": "B", "amount": "18"}),
    ];

    let option = build_echart_option(&ChartType::Bar, &config, &data);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();

    let y_data = parsed["series"][0]["data"].as_array().unwrap();
    assert_eq!(y_data.len(), 2);
    assert!((y_data[0].as_f64().unwrap() - 42.5).abs() < 0.01);
}

#[test]
fn test_empty_data_produces_valid_json() {
    let config = ChartConfig {
        title: "Empty".to_string(),
        x_field: "x".to_string(),
        y_field: "y".to_string(),
        series_field: None,
    };

    let option = build_echart_option(&ChartType::Bar, &config, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&option).unwrap();
    assert!(parsed["xAxis"]["data"].as_array().unwrap().is_empty());
}
