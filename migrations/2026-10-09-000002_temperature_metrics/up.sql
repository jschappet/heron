CREATE TABLE iot_metrics_next (
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
    count BIGINT,
    total_count BIGINT,
    temperature_c DOUBLE,
    humidity_percent DOUBLE,
    reading_json TEXT NOT NULL,
    UNIQUE(host_id, device_id, report_id)
);
INSERT INTO iot_metrics_next (id, host_id, device_id, report_id, boot_id, schema_name, schema_version, observed_at_unix, received_at_unix, window_start_ms, window_end_ms, window_duration_ms, gpio, count, total_count, reading_json) SELECT id, host_id, device_id, report_id, boot_id, schema_name, schema_version, observed_at_unix, received_at_unix, window_start_ms, window_end_ms, window_duration_ms, gpio, count, total_count, reading_json FROM iot_metrics;
DROP TABLE iot_metrics;
ALTER TABLE iot_metrics_next RENAME TO iot_metrics;
CREATE INDEX iot_metrics_device_time ON iot_metrics(host_id, device_id, observed_at_unix);
