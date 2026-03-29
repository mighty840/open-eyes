use open_eyes_core::DbPool;

#[test]
fn test_open_and_init_schema() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    // Verify tables exist
    let tables = pool
        .query_json("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    let table_names: Vec<&str> = tables
        .iter()
        .filter_map(|r| r["name"].as_str())
        .collect();

    assert!(table_names.contains(&"oe_chat_messages"));
    assert!(table_names.contains(&"oe_datasets"));
    assert!(table_names.contains(&"oe_resources"));
}

#[test]
fn test_init_schema_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();
    pool.init_schema().unwrap(); // Should not fail
}

#[test]
fn test_execute_and_query() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute(
        "INSERT INTO oe_datasets (id, title, source_portal) VALUES ('test-1', 'Test Dataset', 'govdata.de')"
    ).unwrap();

    let rows = pool
        .query_json("SELECT id, title FROM oe_datasets WHERE id = 'test-1'")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["id"].as_str().unwrap(), "test-1");
    assert_eq!(rows[0]["title"].as_str().unwrap(), "Test Dataset");
}

#[test]
fn test_query_json_returns_correct_types() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();

    let rows = pool
        .query_json("SELECT 42 AS num, 'hello' AS text, 3.14 AS decimal_val, NULL AS empty_val")
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["num"].as_i64().unwrap(), 42);
    assert_eq!(rows[0]["text"].as_str().unwrap(), "hello");
    assert!((rows[0]["decimal_val"].as_f64().unwrap() - 3.14).abs() < 0.001);
    assert!(rows[0]["empty_val"].is_null());
}

#[test]
fn test_execute_batch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute_batch(
        "INSERT INTO oe_datasets (id, title, source_portal) VALUES ('a', 'A', 'govdata.de');
         INSERT INTO oe_datasets (id, title, source_portal) VALUES ('b', 'B', 'govdata.de');",
    )
    .unwrap();

    let rows = pool
        .query_json("SELECT COUNT(*) AS cnt FROM oe_datasets")
        .unwrap();
    assert_eq!(rows[0]["cnt"].as_i64().unwrap(), 2);
}

#[test]
fn test_get_table_infos_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    let infos = pool.get_table_infos().unwrap();
    assert!(infos.is_empty());
}

#[test]
fn test_get_table_infos_with_loaded_resource() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute(
        "INSERT INTO oe_datasets (id, title, source_portal) VALUES ('ds1', 'Test DS', 'govdata.de')"
    ).unwrap();
    pool.execute(
        r#"INSERT INTO oe_resources (id, dataset_id, url, table_name, row_count, column_names, download_status) VALUES ('r1', 'ds1', 'http://example.com', 'data_ds1_r1', 100, '["col_a", "col_b"]', 'loaded')"#
    ).unwrap();

    // Create the actual data table
    pool.execute("CREATE TABLE data_ds1_r1 (col_a TEXT, col_b INTEGER)")
        .unwrap();

    let infos = pool.get_table_infos().unwrap();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].table_name, "data_ds1_r1");
    assert_eq!(infos[0].dataset_title, "Test DS");
    assert_eq!(infos[0].row_count, 100);
}

#[test]
fn test_get_column_schemas() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();

    pool.execute("CREATE TABLE test_table (name TEXT, age INTEGER, score REAL)")
        .unwrap();

    let schemas = pool
        .get_column_schemas(&["test_table".to_string()])
        .unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].0, "test_table");
    assert_eq!(schemas[0].1.len(), 3);
    assert_eq!(schemas[0].1[0].name, "name");
    assert_eq!(schemas[0].1[1].name, "age");
    assert_eq!(schemas[0].1[2].name, "score");
}

#[test]
fn test_get_column_schemas_nonexistent_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();

    let schemas = pool
        .get_column_schemas(&["nonexistent_table".to_string()])
        .unwrap();
    assert!(schemas.is_empty());
}

#[test]
fn test_chat_messages_insert_and_query() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute(
        "INSERT INTO oe_chat_messages (session_id, role, content) VALUES ('sess1', 'user', 'What data is available?')"
    ).unwrap();
    pool.execute(
        "INSERT INTO oe_chat_messages (session_id, role, content, sql_query) VALUES ('sess1', 'assistant', 'Here are the datasets...', 'SELECT * FROM oe_datasets')"
    ).unwrap();

    let rows = pool
        .query_json(
            "SELECT role, content FROM oe_chat_messages WHERE session_id = 'sess1' ORDER BY id",
        )
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["role"].as_str().unwrap(), "user");
    assert_eq!(rows[1]["role"].as_str().unwrap(), "assistant");
}
