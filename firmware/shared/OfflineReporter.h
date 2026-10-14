#pragma once
#include <Arduino.h>
#include <ArduinoJson.h>
#include <WiFi.h>
#include <Preferences.h>
#include <HTTPClient.h>
#include <NetworkClientSecure.h>
#include <LittleFS.h>
#include <esp_partition.h>
#include <esp_timer.h>
#include <freertos/FreeRTOS.h>
#include <freertos/queue.h>
#include <freertos/task.h>
#include "ReportStore.h"

namespace offline {
inline uint64_t monotonicMs() { return static_cast<uint64_t>(esp_timer_get_time()) / 1000; }
inline bool clockReady() { return time(nullptr) > 1700000000; }
inline bool mountStorage() {
  if (LittleFS.begin(false, "/littlefs", 10, "spiffs")) return true;
  // Initialize ONLY an entirely erased partition. Existing/corrupt filesystems are preserved.
  const esp_partition_t* partition = esp_partition_find_first(ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_DATA_SPIFFS, "spiffs");
  if (!partition) return false;
  uint8_t bytes[256];
  for (size_t offset = 0; offset < partition->size; offset += sizeof(bytes)) {
    size_t length = partition->size - offset < sizeof(bytes) ? partition->size - offset : sizeof(bytes);
    if (esp_partition_read(partition, offset, bytes, length) != ESP_OK) return false;
    for (size_t i = 0; i < length; ++i) if (bytes[i] != 0xff) return false;
    if ((offset % 16384) == 0) delay(1);
  }
  // Arduino's format-on-fail path is now safe because blankness was explicitly verified.
  Serial.println("Initializing blank sensor filesystem.");
  return LittleFS.begin(true, "/littlefs", 10, "spiffs");
}

class Network {
 public:
  bool begin(String& name, const char* defaultName) {
    Preferences preferences;
    preferences.begin("wifi", true);
    ssid_ = preferences.getString("ssid", "");
    password_ = preferences.getString("password", "");
    name = preferences.getString("name", defaultName);
    preferences.end();
    if (ssid_.isEmpty()) return false; // Provision only when credentials are absent.
    WiFi.mode(WIFI_STA);
    WiFi.setAutoReconnect(true);
    WiFi.begin(ssid_.c_str(), password_.c_str());
    attempted_ = millis();
    configTime(0, 0, "pool.ntp.org", "time.nist.gov");
    return true; // Saved credentials, NOT a promise that Wi-Fi is connected yet.
  }
  void tick() {
    if (ssid_.isEmpty()) return;
    if (WiFi.status() == WL_CONNECTED) { wasConnected_ = true; delay_ = 15000; return; }
    if (wasConnected_) { wasConnected_ = false; attempted_ = millis(); delay_ = 15000; }
    if (static_cast<uint32_t>(millis() - attempted_) < delay_) return;
    attempted_ = millis();
    // Connection attempts get at least 15 seconds; retries never erase credentials or reboot.
    WiFi.reconnect();
    delay_ = delay_ < 60000 ? delay_ * 2 : 60000;
    Serial.println("Wi-Fi unavailable; retrying while reports remain buffered.");
  }
 private:
  String ssid_, password_;
  uint32_t attempted_ = 0, delay_ = 15000;
  bool wasConnected_ = false;
};

struct Job { char json[2048]; char reportId[128]; uint64_t sequence; };
struct Result { int status; uint64_t sequence; bool acknowledged; };

class Reporter {
 public:
  Reporter() : store_("/littlefs/sensor-ring-v1"), ring_(store_) {}
  void begin(const String& device, const String& name, uint64_t boot, const char* firmware, const char* version,
             const char* schema, const char* url, const char* token, const char* ca) {
    device_ = device; name_ = name; boot_ = boot; firmware_ = firmware; version_ = version;
    schema_ = schema; url_ = url; token_ = token; ca_ = ca;
    mounted_ = mountStorage();
    if (!mounted_ || !ring_.begin(targetHash(url))) Serial.println("Sensor storage fault: preserving filesystem; reporting disabled.");
    jobs_ = xQueueCreate(1, sizeof(Job)); results_ = xQueueCreate(1, sizeof(Result));
    if (!jobs_ || !results_ || xTaskCreate(worker, "iot-https", 12288, this, 1, nullptr) != pdPASS) {
      if (jobs_) vQueueDelete(jobs_);
      if (results_) vQueueDelete(results_);
      jobs_ = nullptr; results_ = nullptr;
      Serial.println("HTTP worker unavailable; reports will remain buffered.");
    }
  }
  bool append(Record record) {
    record.boot = boot_;
    if (!ring_.append(record)) { ++lostWindows_; return false; }
    Serial.printf("Queued report; pending: %lu, overwritten: %llu\n", (unsigned long)ring_.state().count, (unsigned long long)ring_.state().dropped);
    return true;
  }
  uint64_t pendingTouches() const { return ring_.state().pendingTouches; }
  int lastStatus() const { return lastStatus_; }
  void diagnostics(JsonDocument& doc) {
    doc["wifi_connected"] = WiFi.status() == WL_CONNECTED;
    doc["clock_synchronized"] = clockReady();
    doc["storage_ready"] = mounted_ && ring_.ready();
    doc["queue_depth"] = ring_.state().count;
    doc["queue_capacity"] = Capacity;
    doc["dropped_reports"] = ring_.state().dropped;
    doc["rejected_reports"] = ring_.state().rejected;
    doc["storage_failed_windows_since_boot"] = lostWindows_;
    doc["last_post_status"] = lastStatus_;
    doc["filesystem_bytes_used"] = mounted_ ? LittleFS.usedBytes() : 0;
  }
  String summary() const {
    return String("<p>Buffered reports: ") + String(ring_.state().count) + " / 2048; storage: " +
      (ring_.ready() ? "ready" : "FAULT (data preserved)") + "; overwritten: " +
      String((unsigned long long)ring_.state().dropped) + "; rejected: " + String((unsigned long long)ring_.state().rejected) + "</p>";
  }
  String contents(long requestedPage) {
    String html = "<h2>Buffered readings</h2>";
    if (!ring_.ready()) return html + "<p>Storage unavailable; saved data has been preserved.</p>";
    const uint32_t count = ring_.state().count;
    if (!count) return html + "<p>No readings waiting for delivery.</p>";
    constexpr uint32_t pageSize = 20;
    const uint32_t pages = (count + pageSize - 1) / pageSize;
    const uint32_t page = requestedPage < 0 ? 0 : static_cast<uint32_t>(requestedPage) >= pages ? pages - 1 : requestedPage;
    html += "<p>Oldest first. Page " + String(page + 1) + " of " + String(pages) +
      ". The list changes as reports are delivered.</p>";
    html += "<div style='overflow-x:auto'><table border='1' cellpadding='5'><thead><tr>"
      "<th>Report</th><th>Observed (UTC)</th><th>Time quality</th><th>Window (seconds)</th>"
      "<th>GPIO</th><th>Reading</th></tr></thead><tbody>";
    for (uint32_t index = page * pageSize; index < count && index < (page + 1) * pageSize; ++index) {
      Record record{};
      if (!ring_.inspect(index, record)) {
        html += "<tr><td colspan='6'>Unable to read saved record.</td></tr>";
        break;
      }
      String quality = record.quality == TimeQuality::Pending ? "Awaiting clock" :
        record.quality == TimeQuality::Unknown ? "Unknown" :
        record.quality == TimeQuality::Estimated ? "Estimated" : "Synchronized";
      String observed = "Unknown";
      if (record.observedUnix > 0) {
        time_t timestamp = record.observedUnix;
        struct tm utc{};
        char formatted[24];
        if (gmtime_r(&timestamp, &utc) && strftime(formatted, sizeof(formatted), "%Y-%m-%d %H:%M:%S", &utc)) observed = formatted;
      }
      html += "<tr><td>" + bootName(record.boot) + "-" + String((unsigned long long)record.sequence) +
        "</td><td>" + observed + "</td><td>" + quality + "</td><td>" +
        String(static_cast<uint32_t>(record.windowEnd - record.windowStart) / 1000.0, 1) +
        "</td><td>" + String(record.gpio) + "</td><td>";
      if (record.sensor == Sensor::Touch) html += String(record.count) + " touches (boot total " + String(record.totalCount) + ")";
      else html += String(record.temperature, 2) + " &deg;C; " + String(record.humidity, 2) + "% humidity";
      html += "</td></tr>";
    }
    html += "</tbody></table></div><p>";
    if (page) html += "<a href='/?buffer_page=" + String(page - 1) + "'>Previous</a> ";
    if (page + 1 < pages) html += "<a href='/?buffer_page=" + String(page + 1) + "'>Next</a> ";
    return html + "<a href='/?buffer_page=" + String(page) + "'>Refresh</a></p>";
  }
  void tick() {
    if (!jobs_ || !results_) return;
    Result result{};
    if (xQueueReceive(results_, &result, 0) == pdTRUE) {
      busy_ = false; lastStatus_ = result.status; lastAttempt_ = millis();
      wait_ = result.acknowledged ? 250 : ((result.status == 401 || result.status == 403) ? 300000 : 30000);
      if (result.acknowledged) {
        if (result.sequence) ring_.acknowledge(result.sequence);
        else discoveryDue_ = false;
      } else if (result.sequence && (result.status == 400 || result.status == 409)) {
        // Preserve the last rejected snapshot for diagnosis, count losses, then unblock the queue.
        if (store_.writeRejected(inflight_)) ring_.acknowledge(result.sequence, true);
      }
      Serial.printf("Backend result: %d; acknowledged: %s; queued: %lu\n", result.status, result.acknowledged ? "yes" : "no", (unsigned long)ring_.state().count);
    }
    bool connected = WiFi.status() == WL_CONNECTED;
    String ip = connected ? WiFi.localIP().toString() : "";
    if (connected && (!hadConnection_ || ip != lastIp_)) discoveryDue_ = true;
    hadConnection_ = connected; lastIp_ = ip;
    if (!connected || !clockReady() || busy_ || !ring_.ready() || !strlen(token_) || !strlen(ca_)) return;
    if (static_cast<uint32_t>(millis() - lastAttempt_) < wait_) return;
    Job job{};
    JsonDocument doc;
    doc["device_id"] = device_; doc["name"] = name_; doc["ip_address"] = ip; doc["port"] = 80;
    doc["deployed_code"]["name"] = firmware_; doc["deployed_code"]["version"] = version_;
    auto capabilities = doc["capabilities"].to<JsonArray>();
    capabilities.add(schema_);
    if (!strcmp(schema_, "temperature")) capabilities.add("humidity");
    if (!discoveryDue_) {
      if (!ring_.prepare(inflight_, boot_, monotonicMs(), time(nullptr))) return;
      job.sequence = inflight_.sequence;
      String boot = bootName(inflight_.boot);
      String reportId = boot + "-" + String((unsigned long long)inflight_.sequence);
      reportId.toCharArray(job.reportId, sizeof(job.reportId));
      auto reading = doc["reading"].to<JsonObject>();
      reading["schema"] = inflight_.sensor == Sensor::Touch ? "touch_count" : "temperature";
      reading["schema_version"] = 1; reading["report_id"] = reportId; reading["boot_id"] = boot;
      reading["observed_at_unix"] = inflight_.observedUnix;
      reading["time_quality"] = inflight_.quality == TimeQuality::Unknown ? "unknown" : inflight_.quality == TimeQuality::Estimated ? "estimated" : "synchronized";
      reading["window_start_ms"] = inflight_.windowStart; reading["window_end_ms"] = inflight_.windowEnd;
      reading["window_duration_ms"] = static_cast<uint32_t>(inflight_.windowEnd - inflight_.windowStart);
      reading["data"]["gpio"] = inflight_.gpio;
      if (inflight_.sensor == Sensor::Touch) {
        reading["data"]["count"] = inflight_.count; reading["data"]["total_count"] = inflight_.totalCount;
      } else {
        // Stable rounding matches DHT precision and yields identical retries after a reboot.
        reading["data"]["temperature_c"] = round(inflight_.temperature * 100.0) / 100.0;
        reading["data"]["humidity_percent"] = round(inflight_.humidity * 100.0) / 100.0;
      }
    }
    if (measureJson(doc) >= sizeof(job.json)) { lastStatus_ = -100; return; }
    serializeJson(doc, job.json, sizeof(job.json));
    if (xQueueSend(jobs_, &job, 0) == pdTRUE) { busy_ = true; lastAttempt_ = millis(); }
  }
 private:
  String bootName(uint64_t boot) const {
    char hex[17]; snprintf(hex, sizeof(hex), "%016llx", (unsigned long long)boot);
    return device_ + "-" + hex;
  }
  static void worker(void* context) {
    auto* self = static_cast<Reporter*>(context);
    Job job{};
    for (;;) {
      if (xQueueReceive(self->jobs_, &job, portMAX_DELAY) != pdTRUE) continue;
      Result result{-1, job.sequence, false};
      NetworkClientSecure client; client.setCACert(self->ca_); client.setHandshakeTimeout(8);
      HTTPClient http; http.setConnectTimeout(5000); http.setTimeout(5000);
      if (http.begin(client, self->url_)) {
        http.addHeader("Content-Type", "application/json");
        http.addHeader("Authorization", String("Bearer ") + self->token_);
        result.status = http.POST(String(job.json));
        if (result.status >= 200 && result.status < 300 && http.getSize() >= 0 && http.getSize() <= 4096) {
          JsonDocument response;
          if (!deserializeJson(response, http.getString())) {
            result.acknowledged = response["ok"].is<bool>() && response["ok"].as<bool>() &&
              (!job.sequence || (response["report_id"].is<const char*>() && !strcmp(response["report_id"].as<const char*>(), job.reportId)));
          }
        }
        http.end();
      }
      xQueueSend(self->results_, &result, portMAX_DELAY);
    }
  }
  FileStore store_;
  Ring<FileStore> ring_;
  String device_, name_, lastIp_;
  uint64_t boot_ = 0;
  const char *firmware_, *version_, *schema_, *url_, *token_, *ca_;
  QueueHandle_t jobs_ = nullptr, results_ = nullptr;
  Record inflight_{};
  bool mounted_ = false, busy_ = false, hadConnection_ = false, discoveryDue_ = true;
  uint32_t lastAttempt_ = 0, wait_ = 0, lostWindows_ = 0;
  int lastStatus_ = 0;
};
}
