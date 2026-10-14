#include <WiFi.h>
#include <DHT.h>
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
// DHT11/DHT22 model and data pin are generated from the selected profile.
#define LED_PIN 2
const unsigned long SAMPLE_INTERVAL = 2000UL;

const char* SETUP_SSID = "Sensor-Setup";

 // Add the token provisioned by your backend.
// Generated from firmware config/environment by utils/flash-esp32.
#include "device_config.h"
#include "OfflineReporter.h"
const char* FIRMWARE_NAME = "temperature-webserver";
const char* FIRMWARE_VERSION = "0.2.0";

DHT dht(SENSOR_PIN, SENSOR_MODEL);
Preferences prefs;
WebServer server(80);

String deviceName;
String deviceId;
bool setupMode = false;
unsigned long lastRead = 0;
uint64_t reportStart = 0;
float temperatureC = NAN;
float humidity = NAN;
bool readingValid = false;
uint64_t sampledAtMs = 0;
time_t sampledAtUnix = 0;
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
  doc["gpio"] = SENSOR_PIN;
  doc["reading_valid"] = readingValid;
  if (readingValid) {
    doc["temperature_c"] = temperatureC;
    doc["temperature_f"] = temperatureC * 1.8f + 32.0f;
    doc["humidity_percent"] = humidity;
    doc["observed_at_unix"] = (int64_t)sampledAtUnix;
  } else {
    doc["temperature_c"] = nullptr;
    doc["temperature_f"] = nullptr;
    doc["humidity_percent"] = nullptr;
  }
  reporter.diagnostics(doc);
  doc["report_interval_seconds"] = REPORT_INTERVAL_SECONDS;
  String json;
  serializeJson(doc, json);
  server.send(200, "application/json", json);
}

void showNormalPage() {
  String html = "<!doctype html><html><head><meta name='viewport' content='width=device-width,initial-scale=1'>"
    "<meta http-equiv='refresh' content='5'><title>ESP32 Temperature Sensor</title></head><body>";
  html += "<h1>" + escapeHTML(deviceName) + "</h1>";
  if (readingValid) {
    html += "<p>Temperature: " + String(temperatureC, 1) + " &deg;C / " + String(temperatureC * 1.8f + 32, 1) + " &deg;F</p>";
    html += "<p>Humidity: " + String(humidity, 1) + "%</p>";
  } else html += "<p>Waiting for a valid sensor reading. Check wiring if this persists.</p>";
  html += "<p>IP: " + WiFi.localIP().toString() + "</p>";
  html += "<p>Last POST HTTP status: " + String(reporter.lastStatus()) + "</p>";
  html += reporter.summary();
  html += reporter.contents(server.arg("buffer_page").toInt());
  html += "<p><a href='/sensor'>Sensor JSON</a></p></body></html>";
  server.send(200, "text/html", html);
}

void updateSensor() {
  unsigned long now = millis();
  if (now - lastRead < SAMPLE_INTERVAL) return;
  lastRead = now;
  humidity = dht.readHumidity();
  temperatureC = dht.readTemperature();
  readingValid = isfinite(humidity) && isfinite(temperatureC)
    && humidity >= 0 && humidity <= 100 && temperatureC >= -40 && temperatureC <= 80;
  if (readingValid) {
    sampledAtMs = offline::monotonicMs();
    sampledAtUnix = time(nullptr);
    Serial.printf("Temperature: %.1f C / %.1f F  Humidity: %.1f %%\n", temperatureC, temperatureC * 1.8f + 32, humidity);
  } else Serial.println("DHT read failed. Check sensor model and wiring.");
}

void collectReport() {
  const uint64_t now = offline::monotonicMs();
  if (now - reportStart < static_cast<uint64_t>(REPORT_INTERVAL_SECONDS) * 1000) return;
  offline::Record record{};
  record.windowStart = static_cast<uint32_t>(reportStart);
  record.windowEnd = static_cast<uint32_t>(now);
  record.sensor = offline::Sensor::Temperature;
  record.gpio = SENSOR_PIN;
  if (readingValid && now - sampledAtMs <= SAMPLE_INTERVAL * 2) {
    record.temperature = temperatureC;
    record.humidity = humidity;
    record.observedMs = sampledAtMs;
    record.observedUnix = sampledAtUnix > 1700000000 ? sampledAtUnix : 0;
    record.quality = record.observedUnix ? offline::TimeQuality::Synchronized : offline::TimeQuality::Pending;
    reporter.append(record);
  } else Serial.println("No valid temperature sample for this window; not recording a fabricated value.");
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
  setupMode = !network.begin(deviceName, "Workshop temperature sensor");
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
  dht.begin();
  reportStart = offline::monotonicMs();
}

void loop() {
  server.handleClient();
  updateSensor();
  collectReport();
  network.tick();
  reporter.tick();
  delay(1);
}
