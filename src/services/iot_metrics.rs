use diesel::{
    prelude::*,
    sql_types::{BigInt, Double, Integer, Nullable, Text},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricSchema {
    TouchCount,
    Temperature,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TouchData {
    pub gpio: u8,
    pub count: u32,
    pub total_count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemperatureData {
    pub gpio: u8,
    pub temperature_c: f64,
    pub humidity_percent: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetricData {
    Touch(TouchData),
    Temperature(TemperatureData),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeQuality {
    Synchronized,
    Estimated,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    pub schema: MetricSchema,
    pub schema_version: u32,
    pub report_id: String,
    pub boot_id: String,
    pub observed_at_unix: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_quality: Option<TimeQuality>,
    pub window_start_ms: u32,
    pub window_end_ms: u32,
    pub window_duration_ms: u32,
    pub data: MetricData,
}

impl Reading {
    pub fn is_valid(&self) -> bool {
        let valid_id =
            |s: &str| !s.trim().is_empty() && s.len() <= 160 && !s.chars().any(char::is_control);
        self.schema_version == 1
            && valid_id(&self.report_id)
            && valid_id(&self.boot_id)
            && match self.time_quality {
                Some(TimeQuality::Unknown) => self.observed_at_unix == 0,
                _ => self.observed_at_unix > 0,
            }
            && self.window_duration_ms > 0
            && self.window_end_ms.wrapping_sub(self.window_start_ms) == self.window_duration_ms
            && match (&self.schema, &self.data) {
                (MetricSchema::TouchCount, MetricData::Touch(data)) => {
                    matches!(data.gpio, 0 | 2 | 4 | 12 | 13 | 14 | 15 | 27 | 32 | 33)
                }
                (MetricSchema::Temperature, MetricData::Temperature(data)) => {
                    matches!(
                        data.gpio,
                        0 | 2
                            | 4
                            | 5
                            | 12
                            | 13
                            | 14
                            | 15
                            | 16
                            | 17
                            | 18
                            | 19
                            | 21
                            | 22
                            | 23
                            | 25
                            | 26
                            | 27
                            | 32
                            | 33
                    ) && data.temperature_c.is_finite()
                        && (-40.0..=80.0).contains(&data.temperature_c)
                        && data.humidity_percent.is_finite()
                        && (0.0..=100.0).contains(&data.humidity_percent)
                }
                _ => false,
            }
    }
}

#[derive(Debug, PartialEq)]
pub enum SaveOutcome {
    Created,
    Duplicate,
    Conflict,
}

#[derive(QueryableByName)]
struct StoredReading {
    #[diesel(sql_type = Text)]
    reading_json: String,
}

/// The unique key makes concurrent retries safe; changed readings cannot reuse an ID.
pub fn save(
    conn: &mut SqliteConnection,
    host_id: i32,
    device_id: &str,
    reading: &Reading,
    received_at_unix: i64,
) -> QueryResult<SaveOutcome> {
    let canonical = serde_json::to_string(reading)
        .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
    let (schema, gpio, count, total, temperature, humidity) = match &reading.data {
        MetricData::Touch(data) => (
            "touch_count",
            data.gpio,
            Some(i64::from(data.count)),
            Some(i64::from(data.total_count)),
            None,
            None,
        ),
        MetricData::Temperature(data) => (
            "temperature",
            data.gpio,
            None,
            None,
            Some(data.temperature_c),
            Some(data.humidity_percent),
        ),
    };
    let inserted = diesel::sql_query(
        "INSERT INTO iot_metrics
        (host_id, device_id, report_id, boot_id, schema_name, schema_version,
         observed_at_unix, received_at_unix, window_start_ms, window_end_ms,
         window_duration_ms, gpio, count, total_count, temperature_c, humidity_percent, reading_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(host_id, device_id, report_id) DO NOTHING",
    )
    .bind::<Integer, _>(host_id)
    .bind::<Text, _>(device_id)
    .bind::<Text, _>(&reading.report_id)
    .bind::<Text, _>(&reading.boot_id)
    .bind::<Text, _>(schema)
    .bind::<Integer, _>(reading.schema_version as i32)
    .bind::<BigInt, _>(reading.observed_at_unix)
    .bind::<BigInt, _>(received_at_unix)
    .bind::<BigInt, _>(i64::from(reading.window_start_ms))
    .bind::<BigInt, _>(i64::from(reading.window_end_ms))
    .bind::<BigInt, _>(i64::from(reading.window_duration_ms))
    .bind::<Integer, _>(i32::from(gpio))
    .bind::<Nullable<BigInt>, _>(count)
    .bind::<Nullable<BigInt>, _>(total)
    .bind::<Nullable<Double>, _>(temperature)
    .bind::<Nullable<Double>, _>(humidity)
    .bind::<Text, _>(&canonical)
    .execute(conn)?;
    if inserted == 1 {
        return Ok(SaveOutcome::Created);
    }
    let stored = diesel::sql_query(
        "SELECT reading_json FROM iot_metrics
        WHERE host_id = ? AND device_id = ? AND report_id = ?",
    )
    .bind::<Integer, _>(host_id)
    .bind::<Text, _>(device_id)
    .bind::<Text, _>(&reading.report_id)
    .get_result::<StoredReading>(conn)?;
    Ok(if stored.reading_json == canonical {
        SaveOutcome::Duplicate
    } else {
        SaveOutcome::Conflict
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    fn reading() -> Reading {
        serde_json::from_value(serde_json::json!({
            "schema": "touch_count", "schema_version": 1,
            "report_id": "boot-1", "boot_id": "boot",
            "observed_at_unix": 1791500000,
            "window_start_ms": 0, "window_end_ms": 300000,
            "window_duration_ms": 300000,
            "data": {"gpio": 4, "count": 3, "total_count": 3}
        }))
        .unwrap()
    }

    #[test]
    fn timestamp_quality_is_explicit_and_legacy_json_stays_unchanged() {
        let mut r = reading();
        assert!(r.is_valid());
        assert!(!serde_json::to_string(&r).unwrap().contains("time_quality"));
        r.observed_at_unix = 0;
        assert!(!r.is_valid());
        r.time_quality = Some(TimeQuality::Unknown);
        assert!(r.is_valid());
        r.observed_at_unix = 1791500000;
        assert!(!r.is_valid());
        r.time_quality = Some(TimeQuality::Estimated);
        assert!(r.is_valid());
        r.time_quality = Some(TimeQuality::Synchronized);
        assert!(r.is_valid());
    }

    #[test]
    fn persists_and_deduplicates_per_host_and_device() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute(
            "CREATE TABLE hosts (id INTEGER PRIMARY KEY); INSERT INTO hosts VALUES (1), (2);",
        )
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000000_iot_metrics/up.sql"
        ))
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000002_temperature_metrics/up.sql"
        ))
        .unwrap();
        let mut report = reading();
        assert_eq!(
            save(&mut conn, 1, "device-a", &report, 1).unwrap(),
            SaveOutcome::Created
        );
        assert_eq!(
            save(&mut conn, 1, "device-a", &report, 2).unwrap(),
            SaveOutcome::Duplicate
        );
        assert_eq!(
            save(&mut conn, 2, "device-a", &report, 2).unwrap(),
            SaveOutcome::Created
        );
        assert_eq!(
            save(&mut conn, 1, "device-b", &report, 2).unwrap(),
            SaveOutcome::Created
        );
        if let MetricData::Touch(data) = &mut report.data {
            data.count = 4;
        }
        assert_eq!(
            save(&mut conn, 1, "device-a", &report, 3).unwrap(),
            SaveOutcome::Conflict
        );
        if let MetricData::Touch(data) = &mut report.data {
            data.count = 3;
        }
        assert_eq!(
            save(&mut conn, 1, "device-a", &report, 4).unwrap(),
            SaveOutcome::Duplicate
        );
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000000_iot_metrics/down.sql"
        ))
        .unwrap();
    }

    #[test]
    fn device_listing_updates_urls_and_scopes_metric_pages() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute(
            "CREATE TABLE hosts (id INTEGER PRIMARY KEY); INSERT INTO hosts VALUES (1), (2);",
        )
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000000_iot_metrics/up.sql"
        ))
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000001_iot_devices/up.sql"
        ))
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000002_temperature_metrics/up.sql"
        ))
        .unwrap();
        let mut report = reading();
        save(&mut conn, 1, "a", &report, 1).unwrap();
        assert!(devices(&mut conn, 1).unwrap()[0].sensor_url.is_none());
        save_device(&mut conn, 1, "a", "A", "http://192.168.1.1:80/sensor", 2).unwrap();
        save_device(&mut conn, 1, "a", "A", "http://[::1]:8080/sensor", 3).unwrap();
        save_device(&mut conn, 1, "a", "Old", "http://192.168.1.1:80/sensor", 1).unwrap();
        save_device(
            &mut conn,
            1,
            "discovery-only",
            "B",
            "http://192.168.1.2:80/sensor",
            3,
        )
        .unwrap();
        save_device(
            &mut conn,
            2,
            "private",
            "C",
            "http://192.168.1.3:80/sensor",
            3,
        )
        .unwrap();
        let listed = devices(&mut conn, 1).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(
            listed[0].sensor_url.as_deref(),
            Some("http://[::1]:8080/sensor")
        );
        assert_eq!(devices(&mut conn, 2).unwrap().len(), 1);
        report.report_id = "boot-2".into();
        save(&mut conn, 1, "a", &report, 3).unwrap();
        let first = metrics(&mut conn, 1, "a", i64::MAX, 1).unwrap();
        let second = metrics(&mut conn, 1, "a", first[0].id, 1).unwrap();
        assert_eq!(second.len(), 1);
        assert!(first[0].id > second[0].id);
        assert!(metrics(&mut conn, 2, "a", i64::MAX, 100)
            .unwrap()
            .is_empty());
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000001_iot_devices/down.sql"
        ))
        .unwrap();
    }

    #[test]
    fn temperature_migration_preserves_touch_and_stores_temperature() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        conn.batch_execute(
            "CREATE TABLE hosts (id INTEGER PRIMARY KEY); INSERT INTO hosts VALUES (1);",
        )
        .unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000000_iot_metrics/up.sql"
        ))
        .unwrap();
        conn.batch_execute("INSERT INTO iot_metrics VALUES (1,1,'old','old-report','old-boot','touch_count',1,1,1,0,10,10,4,3,3,'{}');").unwrap();
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000002_temperature_metrics/up.sql"
        ))
        .unwrap();
        assert_eq!(
            metrics(&mut conn, 1, "old", i64::MAX, 100).unwrap().len(),
            1
        );
        let mut report = reading();
        report.schema = MetricSchema::Temperature;
        report.data = MetricData::Temperature(TemperatureData {
            gpio: 4,
            temperature_c: 22.5,
            humidity_percent: 51.0,
        });
        assert!(report.is_valid());
        assert_eq!(
            save(&mut conn, 1, "temperature", &report, 2).unwrap(),
            SaveOutcome::Created
        );
        assert_eq!(
            save(&mut conn, 1, "temperature", &report, 3).unwrap(),
            SaveOutcome::Duplicate
        );
        let rows = metrics(&mut conn, 1, "temperature", i64::MAX, 100).unwrap();
        let decoded: Reading = serde_json::from_str(&rows[0].reading_json).unwrap();
        assert!(decoded.is_valid());
        report.schema = MetricSchema::TouchCount;
        assert!(!report.is_valid());
        report.schema = MetricSchema::Temperature;
        if let MetricData::Temperature(data) = &mut report.data {
            data.humidity_percent = 101.0;
        }
        assert!(!report.is_valid());
        conn.batch_execute(include_str!(
            "../../migrations/2026-10-09-000002_temperature_metrics/down.sql"
        ))
        .unwrap();
        assert_eq!(
            metrics(&mut conn, 1, "temperature", i64::MAX, 100)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn validates_version_ids_and_millis_rollover() {
        let mut report = reading();
        assert!(report.is_valid());
        report.window_start_ms = u32::MAX - 99;
        report.window_end_ms = 100;
        report.window_duration_ms = 200;
        assert!(report.is_valid());
        report.window_duration_ms = 201;
        assert!(!report.is_valid());
        report = reading();
        report.schema_version = 2;
        assert!(!report.is_valid());
        report = reading();
        report.report_id = " ".into();
        assert!(!report.is_valid());
    }
}

#[derive(Debug, QueryableByName, Serialize)]
pub struct DeviceSummary {
    #[diesel(sql_type = Text)]
    pub device_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    pub sensor_url: Option<String>,
    #[diesel(sql_type = BigInt)]
    pub last_seen_unix: i64,
}

pub fn save_device(
    conn: &mut SqliteConnection,
    host_id: i32,
    device_id: &str,
    name: &str,
    sensor_url: &str,
    received: i64,
) -> QueryResult<usize> {
    diesel::sql_query(
        "INSERT INTO iot_devices (host_id, device_id, name, sensor_url, last_seen_unix)
        VALUES (?, ?, ?, ?, ?) ON CONFLICT(host_id, device_id) DO UPDATE SET
        name=excluded.name, sensor_url=excluded.sensor_url, last_seen_unix=excluded.last_seen_unix
        WHERE excluded.last_seen_unix >= iot_devices.last_seen_unix",
    )
    .bind::<Integer, _>(host_id)
    .bind::<Text, _>(device_id)
    .bind::<Text, _>(name)
    .bind::<Text, _>(sensor_url)
    .bind::<BigInt, _>(received)
    .execute(conn)
}

pub fn devices(conn: &mut SqliteConnection, host_id: i32) -> QueryResult<Vec<DeviceSummary>> {
    diesel::sql_query("SELECT device_id, name, sensor_url, last_seen_unix FROM iot_devices WHERE host_id = ?
        UNION ALL SELECT m.device_id, NULL AS name, NULL AS sensor_url, MAX(m.received_at_unix) AS last_seen_unix
        FROM iot_metrics m WHERE m.host_id = ? AND NOT EXISTS
        (SELECT 1 FROM iot_devices d WHERE d.host_id = m.host_id AND d.device_id = m.device_id)
        GROUP BY m.device_id ORDER BY device_id")
        .bind::<Integer,_>(host_id).bind::<Integer,_>(host_id).load(conn)
}

#[derive(QueryableByName)]
pub struct MetricRecord {
    #[diesel(sql_type = BigInt)]
    pub id: i64,
    #[diesel(sql_type = BigInt)]
    pub received_at_unix: i64,
    #[diesel(sql_type = Text)]
    pub reading_json: String,
}

pub fn metrics(
    conn: &mut SqliteConnection,
    host_id: i32,
    device_id: &str,
    before_id: i64,
    limit: i64,
) -> QueryResult<Vec<MetricRecord>> {
    diesel::sql_query(
        "SELECT id, received_at_unix, reading_json FROM iot_metrics
        WHERE host_id = ? AND device_id = ? AND id < ? ORDER BY id DESC LIMIT ?",
    )
    .bind::<Integer, _>(host_id)
    .bind::<Text, _>(device_id)
    .bind::<BigInt, _>(before_id)
    .bind::<BigInt, _>(limit)
    .load(conn)
}
