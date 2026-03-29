use open_eyes_core::DbPool;

/// Test that schema initialization creates all required tables
/// and that the ingest workflow tables are properly linked.
#[test]
fn test_schema_supports_ingestion_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ingest_test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    // Simulate crawl: insert dataset
    pool.execute(
        r#"INSERT INTO oe_datasets (id, title, organization, categories, tags, source_portal)
         VALUES ('ds-001', 'Berlin Population Data', 'Statistisches Bundesamt',
         '["society", "population"]', '["berlin", "demographics"]', 'govdata.de')"#,
    )
    .unwrap();

    // Simulate crawl: insert resources
    pool.execute(
        "INSERT INTO oe_resources (id, dataset_id, name, format, url, download_status) \
         VALUES ('r-001', 'ds-001', 'population.csv', 'CSV', 'https://example.com/pop.csv', 'pending')"
    ).unwrap();
    pool.execute(
        "INSERT INTO oe_resources (id, dataset_id, name, format, url, download_status) \
         VALUES ('r-002', 'ds-001', 'population.json', 'JSON', 'https://example.com/pop.json', 'pending')"
    ).unwrap();

    // Verify pending resources
    let pending = pool
        .query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'pending'")
        .unwrap();
    assert_eq!(pending[0]["cnt"].as_i64().unwrap(), 2);

    // Simulate load: create data table and update resource status
    pool.execute(
        "CREATE TABLE data_ds00001_r000001 (year INTEGER, population INTEGER, city TEXT)",
    )
    .unwrap();
    pool.execute(
        "INSERT INTO data_ds00001_r000001 VALUES (2020, 3700000, 'Berlin'), (2021, 3750000, 'Berlin'), (2022, 3800000, 'Berlin')"
    ).unwrap();
    pool.execute(
        r#"UPDATE oe_resources SET download_status = 'loaded', table_name = 'data_ds00001_r000001',
         row_count = 3, column_names = '["year", "population", "city"]' WHERE id = 'r-001'"#,
    )
    .unwrap();
    pool.execute(
        "UPDATE oe_resources SET download_status = 'error', error_message = 'Connection timeout' WHERE id = 'r-002'"
    ).unwrap();

    // Verify loaded resource shows up in table_infos
    let infos = pool.get_table_infos().unwrap();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].table_name, "data_ds00001_r000001");
    assert_eq!(infos[0].dataset_title, "Berlin Population Data");
    assert_eq!(infos[0].row_count, 3);

    // Verify column schemas
    let schemas = pool
        .get_column_schemas(&["data_ds00001_r000001".to_string()])
        .unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].1.len(), 3);

    // Verify the actual data is queryable
    let data = pool
        .query_json("SELECT city, SUM(population) AS total FROM data_ds00001_r000001 GROUP BY city")
        .unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["city"].as_str().unwrap(), "Berlin");

    // Verify stats queries work
    let stats = pool
        .query_json("SELECT COUNT(*) AS cnt FROM oe_datasets")
        .unwrap();
    assert_eq!(stats[0]["cnt"].as_i64().unwrap(), 1);

    let loaded = pool
        .query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'loaded'")
        .unwrap();
    assert_eq!(loaded[0]["cnt"].as_i64().unwrap(), 1);

    let errors = pool
        .query_json("SELECT COUNT(*) AS cnt FROM oe_resources WHERE download_status = 'error'")
        .unwrap();
    assert_eq!(errors[0]["cnt"].as_i64().unwrap(), 1);
}

/// Test that categories and tags stored as JSON arrays work correctly.
#[test]
fn test_json_array_columns_work() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("arrays_test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute(
        r#"INSERT INTO oe_datasets (id, title, categories, tags, source_portal)
         VALUES ('ds-arr', 'Array Test', '["cat1", "cat2", "cat3"]', '["tag1", "tag2"]', 'govdata.de')"#,
    )
    .unwrap();

    // Test json_each for categories
    let cats = pool
        .query_json("SELECT j.value AS cat FROM oe_datasets d, json_each(d.categories) j WHERE d.id = 'ds-arr'")
        .unwrap();
    assert_eq!(cats.len(), 3);

    // Test json_array_length for tags
    let lens = pool
        .query_json("SELECT json_array_length(tags) AS tag_count FROM oe_datasets WHERE id = 'ds-arr'")
        .unwrap();
    assert_eq!(lens[0]["tag_count"].as_i64().unwrap(), 2);
}

/// Test chat message sequence with auto-incrementing IDs.
#[test]
fn test_chat_message_ordering() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chat_test.sqlite");
    let pool = DbPool::open(&path).unwrap();
    pool.init_schema().unwrap();

    pool.execute(
        "INSERT INTO oe_chat_messages (session_id, role, content) VALUES ('s1', 'user', 'First question')"
    ).unwrap();
    pool.execute(
        "INSERT INTO oe_chat_messages (session_id, role, content, sql_query, chart_spec) \
         VALUES ('s1', 'assistant', 'Answer', 'SELECT 1', '{\"type\":\"bar\"}')",
    )
    .unwrap();
    pool.execute(
        "INSERT INTO oe_chat_messages (session_id, role, content) VALUES ('s1', 'user', 'Follow-up')"
    ).unwrap();

    let messages = pool
        .query_json(
            "SELECT role, content FROM oe_chat_messages WHERE session_id = 's1' ORDER BY id",
        )
        .unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["role"].as_str().unwrap(), "user");
    assert_eq!(messages[1]["role"].as_str().unwrap(), "assistant");
    assert_eq!(messages[2]["content"].as_str().unwrap(), "Follow-up");
}
