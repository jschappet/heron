#include <WiFi.h>
#include <WebServer.h>
#include <Preferences.h>
#include <ArduinoJson.h>
#include <HTTPClient.h>
#include <NetworkClientSecure.h>
#include <time.h>
#include <esp_system.h>
#include <freertos/FreeRTOS.h>
#include <freertos/queue.h>
#include <freertos/task.h>

// Install ArduinoJson (Benoit Blanchon, version 7).
// Capacitive touch on GPIO4: attach a wire/metal pad, not a powered signal.
#if !defined(CONFIG_IDF_TARGET_ESP32)
#error "This sketch targets the original ESP32, not ESP32-S2/S3/C3."
#endif
#define LED_PIN 2   // Change to 33 if using your external LED on D33.
#define TOUCH_PIN 4
const unsigned long TOUCH_SAMPLE_INTERVAL = 20;
const unsigned long TOUCH_DEBOUNCE = 60;
// On original ESP32, touch lowers the raw reading. Adjust these ratios if needed.
const float TOUCH_RATIO = 0.70f;
const float RELEASE_RATIO = 0.85f;

const char* SETUP_SSID = "Sensor-Setup";

 // Add the token provisioned by your backend.
// Generated from firmware config/environment by utils/flash-esp32.
#include "device_config.h"
#include "OfflineReporter.h"
const char* FIRMWARE_NAME = "touch-counter-webserver";
const char* FIRMWARE_VERSION = "0.3.0";

Preferences prefs;
WebServer server(80);

String deviceName;
String deviceId;
bool setupMode = false;
unsigned long lastRead = 0;
uint64_t reportStart = 0;
unsigned long candidateSince = 0;
uint32_t touchTotal = 0;
uint32_t lastQueuedTotal = 0;
uint32_t touchRaw = 0;
uint32_t touchBaseline = 0;
bool touched = false;
bool candidateTouched = false;
String bootId;
uint64_t bootNumber;
offline::Network network;
offline::Reporter reporter;

String escapeHTML(String text) {
  text.replace("&", "&amp;");
  text.replace("<", "&lt;");
  text.replace(">", "&gt;");
  text.replace("\"", "&quot;");
  text.replace("'", "&#39;");
  return text;
}

void saveCredentials(String ssid, String password, String name) {
  prefs.begin("wifi", false);
  prefs.putString("ssid", ssid);
  prefs.putString("password", password);
  prefs.putString("name", name);
  prefs.end();
}

void showSetupPage() {
  String html = R"HTML(<!doctype html><html><head>
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>ESP32 Setup</title><style>
  body{font-family:sans-serif;max-width:400px;margin:40px auto;padding:20px}
  input,button{width:100%;padding:12px;margin:8px 0 20px;box-sizing:border-box}
  </style></head><body><h1>ESP32 Setup</h1>
  <form action="/save" method="POST"><label>Device name</label>
  <input name="name" maxlength="80" required value=")HTML";
  html += escapeHTML(deviceName);
  html += R"HTML("><label>WiFi Network</label>
  <input name="ssid" maxlength="32" required><label>Password</label>
  <input type="password" name="password" maxlength="64">
  <button>Save &amp; Connect</button></form></body></html>)HTML";
  server.send(200, "text/html", html);
}

void handleSave() {
  if (!setupMode) { server.send(403, "text/plain", "Setup mode required."); return; }
  String ssid = server.arg("ssid");
  String password = server.arg("password");
  String name = server.arg("name");
  name.trim();
  if (ssid.isEmpty() || ssid.length() > 32 || password.length() > 64 ||
      name.isEmpty() || name.length() > 80) {
    server.send(400, "text/plain", "Check network, password, and device name.");
    return;
  }
  saveCredentials(ssid, password, name);
  server.send(200, "text/html", "<h1>Saved!</h1><p>Restarting to connect to WiFi.</p>");
  delay(2000);
  ESP.restart();
}

void showSensorJSON() {
  JsonDocument doc;
  doc["device_id"] = deviceId;
  doc["name"] = deviceName;
  doc["boot_id"] = bootId;
  doc["gpio"] = TOUCH_PIN;
  doc["touch_raw"] = touchRaw;
  doc["touch_baseline"] = touchBaseline;
  doc["touched"] = touched;
  doc["count"] = reporter.pendingTouches() + static_cast<uint32_t>(touchTotal - lastQueuedTotal);
  doc["total_count"] = touchTotal;
  doc["unacknowledged_count"] = reporter.pendingTouches() + static_cast<uint32_t>(touchTotal - lastQueuedTotal);
  reporter.diagnostics(doc);
  doc["report_interval_seconds"] = REPORT_INTERVAL_SECONDS;
  String json;
  serializeJson(doc, json);
  server.send(200, "application/json", json);
}

void showNormalPage() {
  String html = "<!doctype html><html><head><meta name='viewport' content='width=device-width,initial-scale=1'>"
    "<meta http-equiv='refresh' content='5'><title>ESP32 Touch Counter</title></head><body>";
  html += "<h1>" + escapeHTML(deviceName) + "</h1>";
  html += "<p>GPIO4 touches awaiting successful report: " + String(reporter.pendingTouches() + static_cast<uint32_t>(touchTotal - lastQueuedTotal)) + "</p>";
  html += "<p>Raw: " + String(touchRaw) + "; baseline: " + String(touchBaseline) + "</p>";
  html += String("<p>Touch state: ") + (touched ? "touched" : "released") + "</p>";
  html += "<p>IP: " + WiFi.localIP().toString() + "</p>";
  html += "<p>Last POST HTTP status: " + String(reporter.lastStatus()) + "</p>";
  html += reporter.summary();
  html += reporter.contents(server.arg("buffer_page").toInt());
  html += "<p><a href='/sensor'>Sensor JSON</a></p></body></html>";
  server.send(200, "text/html", html);
}

void initializeTouch() {
  Serial.println("Calibrating GPIO4: keep your hand OFF the wire/pad for 2 seconds.");
  delay(1000);
  uint64_t sum = 0;
  for (int i = 0; i < 50; ++i) {
    sum += touchRead(TOUCH_PIN);
    delay(20);
  }
  touchBaseline = sum / 50;
  Serial.printf("Touch baseline: %lu; touch below %.1f; release above %.1f\n",
    (unsigned long)touchBaseline, touchBaseline * TOUCH_RATIO, touchBaseline * RELEASE_RATIO);
  if (!touchBaseline) Serial.println("Invalid baseline: check pad/wire and restart.");
  reportStart = offline::monotonicMs();
  candidateSince = millis();
}

void updateSensor() {
  unsigned long now = millis();
  if (now - lastRead < TOUCH_SAMPLE_INTERVAL) return;
  lastRead = now;
  touchRaw = touchRead(TOUCH_PIN);
  if (!touchBaseline) return;
  // Hysteresis prevents noisy values near the threshold from counting twice.
  bool detected = touched ? touchRaw < touchBaseline * RELEASE_RATIO
                          : touchRaw < touchBaseline * TOUCH_RATIO;
  if (detected != candidateTouched) {
    candidateTouched = detected;
    candidateSince = now;
  }
  if (candidateTouched != touched && now - candidateSince >= TOUCH_DEBOUNCE) {
    touched = candidateTouched;
    if (touched) {
      ++touchTotal;
      Serial.printf("Touch #%llu (raw %lu)\n", (unsigned long long)(reporter.pendingTouches() + static_cast<uint32_t>(touchTotal - lastQueuedTotal)), (unsigned long)touchRaw);
    }
  }
}

void collectReport() {
  const uint64_t now = offline::monotonicMs();
  if (now - reportStart < static_cast<uint64_t>(REPORT_INTERVAL_SECONDS) * 1000) return;
  offline::Record record{};
  record.windowStart = static_cast<uint32_t>(reportStart);
  record.windowEnd = static_cast<uint32_t>(now);
  record.sensor = offline::Sensor::Touch;
  record.gpio = TOUCH_PIN;
  record.count = static_cast<uint32_t>(touchTotal - lastQueuedTotal);
  record.totalCount = touchTotal;
  record.observedMs = now;
  record.observedUnix = offline::clockReady() ? time(nullptr) : 0;
  record.quality = record.observedUnix ? offline::TimeQuality::Synchronized : offline::TimeQuality::Pending;
  reporter.append(record);
  // A storage fault is visible in diagnostics; do not silently extend the next interval.
  lastQueuedTotal = touchTotal;
  reportStart = now;
}

void setup() {
  Serial.begin(115200);
  // Keep the GPIO2 status LED off; the separate power LED is hardware-controlled.
  digitalWrite(LED_PIN, LOW);
  pinMode(LED_PIN, OUTPUT);
  // Hardware-derived identity remains stable across WiFi changes/reboots.
  uint64_t chip = ESP.getEfuseMac();
  char id[32];
  snprintf(id, sizeof(id), "esp32-%04x%08x",
    (unsigned int)((chip >> 32) & 0xffff), (unsigned int)(chip & 0xffffffff));
  deviceId = id;
  bootNumber = (static_cast<uint64_t>(esp_random()) << 32) | esp_random();
  char boot[24];
  snprintf(boot, sizeof(boot), "%016llx", (unsigned long long)bootNumber);
  bootId = deviceId + "-" + String(boot);
  Serial.println("ESP32 starting: " + deviceId);
  setupMode = !network.begin(deviceName, "Workshop touch counter");
  if (setupMode) {
    WiFi.mode(WIFI_AP);
    WiFi.softAP(SETUP_SSID);
    Serial.println("Connect to Sensor-Setup, then http://" + WiFi.softAPIP().toString());
    server.on("/", HTTP_GET, showSetupPage);
    server.on("/save", HTTP_POST, handleSave);
  } else server.on("/", HTTP_GET, showNormalPage);
  server.on("/sensor", HTTP_GET, showSensorJSON);
  reporter.begin(deviceId, deviceName, bootNumber, FIRMWARE_NAME, FIRMWARE_VERSION,
    SENSOR_TYPE, DISCOVERY_URL, DEVICE_TOKEN, ROOT_CA);
  server.begin();
  initializeTouch();
}

void loop() {
  server.handleClient();
  updateSensor();
  collectReport();
  network.tick();
  reporter.tick();
  delay(1);
}
