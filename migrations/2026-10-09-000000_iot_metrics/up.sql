CREATE TABLE iot_metrics (
    id INTEGER PRIMARY KEY NOT NULL,
    host_id INTEGER NOT NULL REFERENCES hosts(id),
    device_id TEXT NOT NULL,
    report_id TEXT NOT NULL,
    boot_id TEXT NOT NULL,
    schema_name TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    observed_at_unix BIGINT NOT NULL,
    received_at_unix BIGINT NOT NULL,
    window_start_ms BIGINT NOT NULL,
    window_end_ms BIGINT NOT NULL,
    window_duration_ms BIGINT NOT NULL,
    gpio INTEGER NOT NULL,
    count BIGINT NOT NULL,
    total_count BIGINT NOT NULL,
    reading_json TEXT NOT NULL,
    UNIQUE(host_id, device_id, report_id)
);
CREATE INDEX iot_metrics_device_time ON iot_metrics(host_id, device_id, observed_at_unix);
