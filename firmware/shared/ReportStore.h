#pragma once
#include "ReportRing.h"
#include <cstdio>
#include <unistd.h>
#include <sys/stat.h>
#include <string>

namespace offline {
// 64 records per segment: bounded copy-on-write work instead of rewriting the whole ring.
class FileStore {
 public:
  explicit FileStore(const char* directory) : directory_(directory) {}
  bool pristine() {
    struct stat info{};
    if (stat(directory_.c_str(), &info) == 0) return false;
    return mkdir(directory_.c_str(), 0700) == 0;
  }
  bool readState(unsigned index, State& value) { return read(path("meta", index), &value, sizeof(value), 0); }
  bool writeState(unsigned index, const State& value) { return atomic(path("meta", index), &value, sizeof(value)); }
  bool writeRejected(const Record& value) { return atomic(directory_ + "/last-rejected", &value, sizeof(value)); }
  bool readFlight(Record& value) { return read(directory_ + "/inflight", &value, sizeof(value), 0); }
  bool writeFlight(const Record& value) { return atomic(directory_ + "/inflight", &value, sizeof(value)); }
  bool readSlot(unsigned index, Record& value) { return read(path("segment", index / 64), &value, sizeof(value), (index % 64) * sizeof(value)); }
  bool writeSlot(unsigned index, const Record& value) {
    std::string name = path("segment", index / 64);
    FILE* file = fopen(name.c_str(), "r+b");
    if (!file) file = fopen(name.c_str(), "w+b");
    if (!file) return false;
    bool ok = fseek(file, (index % 64) * sizeof(value), SEEK_SET) == 0 && fwrite(&value, 1, sizeof(value), file) == sizeof(value);
    if (ok) ok = fflush(file) == 0 && fsync(fileno(file)) == 0;
    if (fclose(file) != 0) ok = false;
    return ok;
  }
 private:
  std::string path(const char* name, unsigned index) { return directory_ + "/" + name + "." + std::to_string(index); }
  bool read(const std::string& name, void* value, size_t size, size_t offset) {
    FILE* file = fopen(name.c_str(), "rb");
    if (!file) return false;
    bool ok = fseek(file, offset, SEEK_SET) == 0 && fread(value, 1, size, file) == size;
    fclose(file); return ok;
  }
  bool atomic(const std::string& name, const void* value, size_t size) {
    std::string temp = name + ".tmp";
    FILE* file = fopen(temp.c_str(), "wb");
    if (!file) return false;
    bool ok = fwrite(value, 1, size, file) == size;
    if (ok) ok = fflush(file) == 0 && fsync(fileno(file)) == 0;
    if (fclose(file) != 0) ok = false;
    return ok && rename(temp.c_str(), name.c_str()) == 0;
  }
  std::string directory_;
};
}
