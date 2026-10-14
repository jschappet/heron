CREATE TABLE iot_devices (
    host_id INTEGER NOT NULL REFERENCES hosts(id),
    device_id TEXT NOT NULL,
    name TEXT NOT NULL,
    sensor_url TEXT NOT NULL,
    last_seen_unix BIGINT NOT NULL,
    PRIMARY KEY (host_id, device_id)
);
