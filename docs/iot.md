# IoT metrics and Arduino development

## Server

POST to the existing IoT URL (configured API context followed by `/iot`),
with `Authorization: Bearer <device token>` and `Content-Type: application/json`.
Configuration uses `[web_config].iot_device_token` and `iot_storage_dir`.
Discovery-only requests retain their existing behavior. Touch reports add `reading`:

```json
{
  "device_id": "esp32-example",
  "name": "Touch counter",
  "ip_address": "192.168.1.10",
  "port": 80,
  "deployed_code": {"name": "touch-counter-webserver", "version": "0.2.0"},
  "capabilities": ["touch_count"],
  "reading": {
    "schema": "touch_count",
    "schema_version": 1,
    "report_id": "boot-example-1",
    "boot_id": "boot-example",
    "observed_at_unix": 1791500000,
    "window_start_ms": 0,
    "window_end_ms": 300000,
    "window_duration_ms": 300000,
    "data": {"gpio": 4, "count": 3, "total_count": 3}
  }
}
```

Startup migrations create `iot_metrics` in SQLite. Rows belong to the host resolved
by middleware, with device/report IDs, observation and receipt times, window
values, GPIO, interval count and cumulative count. Both `touch_count` and `temperature` version 1
are supported. Invalid/unsupported readings return 400. Window duration must match
unsigned 32-bit elapsed time, including rollover, and be positive. Counts use
unsigned 32-bit values, matching the firmware.

Success returns 201 with the existing `registration_file` and, for a reading,
`report_id` and `metric_duplicate`. Repeating the same host/device/report ID and
reading succeeds without inserting another metric. Changed reading content under
the same ID returns 409.

Raw JSON is archived before database persistence. Database failures return 500
so the device retries; files can therefore exist for unsuccessful requests.
Retries may create additional raw files, but cannot duplicate metric rows. A 2xx
response for a reading means both writes succeeded. Historical archived readings
are not automatically backfilled.

Example query (bind the actual host and device):

```sql
SELECT observed_at_unix, count, total_count, window_duration_ms
FROM iot_metrics
WHERE host_id = ? AND device_id = ?
ORDER BY observed_at_unix;
```

## VS Code

### Flash from a terminal

From the Heron directory:

```sh
node utils/flash-esp32 --list
node utils/flash-esp32 --sensor touch --env DEV /dev/cu.usbserial-0001
```

Use the actual ESP32 port shown by `--list`. Close serial monitors first.
The script compiles into a fresh directory under `target/arduino-touch`, uploads
only after a successful build, and resets the board. It stops on any error.
Override `ARDUINO_CLI`, `ARDUINO_SKETCH`, or `ARDUINO_FQBN` for other installations
or board configurations. The VS Code upload task uses this same script.

### Workspace

Open `heron-arduino.code-workspace`. It includes Heron and the existing sketch at
`firmware/esp32_sensor` inside this repository. Both compile and upload tasks use
the same configuration-aware flashing script.

Use **Tasks: Run Task** in the Command Palette:

1. **Arduino: List boards** lists serial ports.
2. **Arduino: Compile** builds the sketch (also the default build task).
3. **Arduino: Upload (compiles first)** builds and flashes the selected port.
4. **Arduino: Serial monitor** opens the selected port at 115200 baud. Stop the
   monitor before uploading again.

Tasks prompt for the board identifier, defaulting to `esp32:esp32:esp32` (original
ESP32 Dev Module). Match the board and options selected in Arduino IDE. The sketch
rejects S2/S3/C3 targets. Select the actual ESP32 serial port from the board list.

Tasks use this Mac's CLI bundled in `/Applications/Arduino IDE.app` and its existing
ESP32 core and ArduinoJson installation. On another machine, change the command
to the installed `arduino-cli` path and install the core/libraries. Build outputs
use Heron's ignored `target/arduino-touch`. Rust and C++ editor extensions are
recommended by the workspace, but are not installed automatically.

References: [Arduino CLI workflow](https://docs.arduino.cc/arduino-cli/getting-started/)
and [ESP32 installation](https://docs.espressif.com/projects/arduino-esp32/en/latest/installing.html).

## Reading devices and metrics

Both GET routes use the same bearer token as POST and are scoped to the request's
host. Prefix the paths below with the configured API context, if any.

- `GET /iot/devices` returns `{"devices": [...]}` with `device_id`, `name`,
  `sensor_url`, and `last_seen_unix` for each device. The sensor URL points to the
  device's HTTP `/sensor` JSON endpoint, for example `http://192.168.1.152:80/sensor`.
  It is a LAN address, reachable only from a network that can access that device.
  Every discovery/report updates the device's address and last-seen time.
- `GET /iot/devices/esp32-fc01e40d5020/metrics?limit=100` returns `device_id`,
  `metrics`, and `next_before_id`. Each metric contains `id`, `received_at_unix`,
  and the parsed `reading` object. Results are newest insertion first. Follow
  `?before_id=<next_before_id>&limit=100` for the next page; null means no more.
  Default limit is 100, maximum 500. Unknown devices return an empty metric list.

The new device-table migration runs at startup. Devices with existing metrics but
no new registration are listed with null name/URL until their next successful
POST. Discovery-only devices are also listed, even before their first metric.


## Firmware configuration

The flasher selects `firmware/config/<sensorType>.<env>.json` using explicit
sensor and environment selections. `DEV` and `PROD` are case-insensitive; the
filenames use lowercase. Flags override the `SENSOR_TYPE` and `FLASH_ENV`
environment variables. Neither selection defaults silently.

```sh
node utils/flash-esp32 --sensor touch --env DEV --compile-only
node utils/flash-esp32 --sensor touch --env PROD /dev/cu.usbserial-0001
SENSOR_TYPE=touch FLASH_ENV=DEV node utils/flash-esp32 /dev/cu.usbserial-0001
```

These select `touch.dev.json` or `touch.prod.json`. The existing local settings
were copied to `touch.dev.json`. Production settings have not been assumed:
copy `touch.prod.example.json` to `touch.prod.json` and fill in the correct values.
Actual config files are ignored by Git; examples are tracked. The old
`firmware/config.local.json` is retained but no longer used by the script.

Each config provides `DISCOVERY_URL` (full HTTPS endpoint including path),
`DEVICE_TOKEN`, and either `ROOT_CA_FILE` (relative to the config file) or
`ROOT_CA` (inline PEM text). Environment variables with those names may override
the selected file's values. Environment certificate paths are relative to the
current directory. `ESP32_CONFIG_DIR` overrides the config directory; the selected
file must exist, and there is no fallback to another profile.

`touch` selects the touch firmware and its `touch_count` schema.
`temperature` selects `firmware/esp32_temperature` and its `temperature` schema.
Unsupported sensor types are rejected before compiling or uploading. Do not put the profile selector
`SENSOR_TYPE=touch` inside JSON; the selector comes from the command or environment.

VS Code compile/upload tasks prompt for both selections. Generated headers and
build artifacts remain under ignored `target/`; credentials are embedded in the
firmware binary, so keep those artifacts private.

The flasher requires Node.js 18 or later and Arduino CLI. It uses only built-in
Node modules; no package installation is required. The old shell entry point
forwards to Node for compatibility. Run its tests with
`node --test utils/flash-esp32/index.test.js`.


## Temperature and humidity sensor

`firmware/esp32_temperature/esp32_temperature.ino` follows the touch firmware's
Wi-Fi provisioning, HTTPS discovery, device/boot identity, and retry workflow.
Install Adafruit's DHT sensor library and Adafruit Unified Sensor (already present
on this development machine). Create `firmware/config/temperature.dev.json` from
its example and set the actual endpoint/token/CA, `SENSOR_MODEL` (`DHT11` or `DHT22`),
and `SENSOR_PIN` (actual data GPIO). The example leaves the pin null intentionally;
compilation fails until it is supplied. Both hardware settings can also be supplied
as environment variables. These pins target an original ESP32, like the touch board.

```sh
node utils/flash-esp32 --sensor temperature --env DEV --compile-only
node utils/flash-esp32 --sensor temperature --env DEV /dev/cu.usbserial-0001
```

Samples are read every two seconds. `/sensor` provides temperature in Celsius and
Fahrenheit, humidity percentage and `reading_valid`. Invalid reads return null
values. The device sends its latest valid sample every five minutes, using the
sample's observation timestamp. Each reading contains `schema: "temperature"`,
version 1, report/boot IDs, window timing, and
`data: {gpio, temperature_c, humidity_percent}`. It does not calculate a five-minute
average. A window with no fresh valid sample is skipped; invalid values are never sent as zero. Discovery can still succeed while the sensor is unavailable.

Completed reporting windows are persisted in the shared flash-backed ring described below.
Two-second samples are not individually retained.

Heron's temperature migration adds nullable temperature/humidity columns and makes
count columns nullable (temperature rows have no touch counts). Existing touch
rows and their report IDs are preserved. GET metric responses retain the same
wrapper and include the temperature reading. Restart Heron to apply the migration.


## Install the command locally

From the Heron repository root, run `npm link` once. This exposes the package's
`flash-esp32` bin through your npm global bin directory (which must be on PATH).
It links this checkout, so source changes are available without relinking.

```sh
npm link
flash-esp32 --help
flash-esp32 --sensor temperature --env DEV /dev/cu.usbserial-0001
```

Config and sketch paths are resolved relative to the linked Heron checkout, so the
command works from other directories. `npm unlink -g heron` removes the link.

## Offline reporting (touch 0.3.0 / temperature 0.2.0)

Both sketches use `firmware/shared` (staged automatically by the Node flasher).
`REPORT_INTERVAL_SECONDS` defaults to 300; set it in `<sensor>.<env>.json` or as
an environment override. Accepted values are integer seconds from 5 to 86400.
For example, add `"REPORT_INTERVAL_SECONDS": 300` to your profile. Short intervals
are useful for bench tests but increase flash writes and reduce retained history.

The queue retains 2,048 completed reporting windows: 7 days, 2 hours, 40 minutes
at the default cadence. It uses 2,049 fixed 68-byte slots (one spare for safe
append), segmented files, CRCs, and alternating metadata snapshots. Raw slots
use about 136 KiB; LittleFS metadata and copy-on-write need additional space.
The existing 1.375 MiB `spiffs` partition is mounted as LittleFS; the app/OTA
partition layout is unchanged. First boot formats only an entirely erased
partition. A nonblank filesystem that cannot be mounted, corrupt queue, or
changed discovery URL disables reporting and preserves data for diagnosis.
Changing DEV/PROD therefore cannot silently upload a backlog to another server.

Collection continues while Wi-Fi or HTTPS is unavailable. Saved Wi-Fi credentials
trigger connection retries with 15/30/60-second backoff; the setup AP is entered
only when credentials are missing. HTTPS runs on a separate worker. Flash writes
still occur on the collection loop and may briefly delay sampling.

Reports drain oldest first, about 250 ms apart after acknowledgments. Network or
server errors retry after 30 seconds; authentication errors wait five minutes.
Only a successful JSON acknowledgment with the matching report ID removes a
report. A permanent 400/409 response saves the latest rejected record, increments
a persistent rejection counter, and advances the queue. When full, the oldest
unreported record is overwritten and the persistent dropped counter increases.
This is bounded storage, not a guarantee of lossless delivery.

Completed windows survive normal restarts; a partial window can be lost at power
failure. A failed storage write stops queue mutations and increments a diagnostic
counter for missed windows. Touch counts awaiting delivery decrease when their
reports are acknowledged (or explicitly dropped/rejected); touches collected
while delivery is in progress remain pending. Boot totals remain separate.

Reports now include `time_quality`: `synchronized`, `estimated`, or `unknown`.
After time is initially obtained from NTP, the device clock runs through outages.
If time was unavailable at collection, the same boot's monotonic elapsed time can
estimate it later. If the device rebooted before that was possible, the record
uses `observed_at_unix: 0` and `time_quality: "unknown"`. Heron accepts zero only
with that explicit quality; legacy reports without quality still require a
positive timestamp. Unknown times remain available in the stored JSON but must
not be plotted as actual 1970 observations. A snapshot is persisted before the
first POST, keeping retry IDs, values, and timestamps identical across restarts.

`/sensor` exposes queue depth/capacity, dropped/rejected counts, storage status,
missed windows since boot, Wi-Fi/clock status, and last HTTP status. The sensor
home page also displays queue health. GPIO2 stays low; there is no heartbeat blink.

### Verification

Run `node --test utils/flash-esp32/index.test.js` for flasher configuration checks.
Build/run `firmware/tests/report-ring.cpp` with a C++17 compiler for FIFO,
overflow, interrupted metadata commits, reopen, timestamp freezing, corruption,
and segmented file tests. Run `cargo test services::iot_metrics` for ingestion.
Compile both sketches with the flasher's `--compile-only` option.

Hardware acceptance remains necessary: start with the default partition layout,
verify initialization and queue diagnostics, disconnect Wi-Fi for several windows,
restore it and confirm FIFO delivery, then reboot with a backlog. Repeat with
power removed during writes and HTTPS delivery. Check reset reasons and filesystem
health. Host tests do not substitute for power-cut or overnight tests on ESP32.

### Viewing buffered readings on the sensor

The home page on both sensors lists queued reports oldest first, 20 per page,
with Previous/Next links. Each row shows report ID, saved observation time (UTC),
time quality, window duration, GPIO, and touch counts or temperature/humidity.
Pending/unknown timestamps are explicitly labeled. The view uses the frozen
retry timestamp for an in-flight report, when present. Viewing does not modify,
acknowledge, or remove records. Page contents can shift as delivery drains the
queue; use Refresh to see current contents. Pagination bounds page memory even
when all 2,048 slots are occupied. Storage faults and empty queues have explicit
messages.
