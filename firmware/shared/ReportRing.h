#pragma once
#include <cstdint>
#include <cstddef>
#include <cstring>

namespace offline {
constexpr uint32_t Capacity = 2048;
constexpr uint32_t Slots = Capacity + 1; // Spare slot keeps the committed head intact during append.
constexpr uint32_t Magic = 0x524E4731;
enum class Sensor : uint8_t { Touch = 1, Temperature = 2 };
enum class TimeQuality : uint8_t { Pending = 0, Synchronized = 1, Estimated = 2, Unknown = 3 };

// Fixed-width packed on-disk format. Change version/magic for incompatible layouts.
#pragma pack(push, 1)
struct Record {
  uint32_t magic = Magic;
  uint64_t sequence = 0;
  uint64_t boot = 0;
  uint64_t observedMs = 0;
  int64_t observedUnix = 0;
  uint32_t windowStart = 0;
  uint32_t windowEnd = 0;
  uint32_t count = 0;
  uint32_t totalCount = 0;
  float temperature = 0;
  float humidity = 0;
  Sensor sensor = Sensor::Touch;
  TimeQuality quality = TimeQuality::Pending;
  uint8_t gpio = 0;
  uint8_t reserved = 0;
  uint32_t checksum = 0;
};
struct State {
  uint32_t magic = Magic;
  uint64_t generation = 0;
  uint64_t target = 0;
  uint64_t nextSequence = 1;
  uint64_t dropped = 0;
  uint64_t rejected = 0;
  uint64_t pendingTouches = 0;
  uint32_t head = 0;
  uint32_t count = 0;
  uint32_t checksum = 0;
};
#pragma pack(pop)
static_assert(sizeof(Record) == 68, "Record format changed");
inline uint32_t crc(const void* bytes, size_t size) {
  auto p = static_cast<const uint8_t*>(bytes);
  uint32_t value = 0xffffffff;
  while (size--) {
    value ^= *p++;
    for (int i = 0; i < 8; ++i) value = (value >> 1) ^ (0xedb88320U & (0U - (value & 1)));
  }
  return ~value;
}
template<class T> void seal(T& value) { value.checksum = crc(&value, sizeof(T) - 4); }
template<class T> bool valid(const T& value) { return value.magic == Magic && value.checksum == crc(&value, sizeof(T) - 4); }
inline uint64_t targetHash(const char* text) {
  uint64_t hash = 14695981039346656037ULL;
  while (*text) { hash ^= static_cast<uint8_t>(*text++); hash *= 1099511628211ULL; }
  return hash;
}
inline Record resolveTime(Record record, uint64_t boot, uint64_t nowMs, int64_t nowUnix) {
  if (record.quality != TimeQuality::Pending) return record;
  if (record.boot == boot && nowMs >= record.observedMs && nowUnix > 1700000000) {
    record.observedUnix = nowUnix - static_cast<int64_t>((nowMs - record.observedMs) / 1000);
    record.quality = TimeQuality::Estimated;
  } else {
    record.observedUnix = 0; // Explicit unknown, never represented as the upload time.
    record.quality = TimeQuality::Unknown;
  }
  seal(record);
  return record;
}

// Store implements bounded slot I/O and atomic metadata/in-flight replacement.
template<class Store> class Ring {
 public:
  explicit Ring(Store& store) : store_(store) {}
  bool begin(uint64_t target) {
    State a{}, b{};
    bool va = store_.readState(0, a) && valid(a);
    bool vb = store_.readState(1, b) && valid(b);
    if (!va && !vb) {
      if (!store_.pristine()) return fault();
      state_ = State{}; state_.target = target;
      if (!commit(state_)) return false;
    } else state_ = (!vb || (va && a.generation > b.generation)) ? a : b;
    if (state_.target != target || state_.count > Capacity || state_.head >= Slots) return fault();
    // Fail closed if committed records are corrupt. Never silently format or clear them.
    uint64_t pending = 0;
    for (uint32_t i = 0; i < state_.count; ++i) {
      Record r{};
      if (!store_.readSlot((state_.head + i) % Slots, r) || !valid(r)) return fault();
      if (r.sequence != state_.nextSequence - state_.count + i) return fault();
      if (r.sensor == Sensor::Touch) pending += r.count;
    }
    if (pending != state_.pendingTouches) return fault();
    ready_ = true; return true;
  }
  bool append(Record record) {
    if (!ready_) return false;
    State next = state_;
    if (next.count == Capacity) {
      Record oldest{};
      if (!peek(oldest)) return false;
      if (oldest.sensor == Sensor::Touch) next.pendingTouches -= oldest.count;
      next.head = (next.head + 1) % Slots; --next.count; ++next.dropped;
    }
    record.sequence = next.nextSequence++;
    seal(record);
    if (!store_.writeSlot((next.head + next.count) % Slots, record)) return fault();
    ++next.count;
    if (record.sensor == Sensor::Touch) next.pendingTouches += record.count;
    return commit(next);
  }
  bool peek(Record& record) {
    if (!ready_ || !state_.count) return false;
    if (!store_.readSlot(state_.head, record) || !valid(record)) return fault();
    return true;
  }
  // Read-only inspection: never freezes timestamps, acknowledges, or changes queue state.
  bool inspect(uint32_t offset, Record& record) {
    if (!ready_ || offset >= state_.count) return false;
    if (!store_.readSlot((state_.head + offset) % Slots, record) || !valid(record) ||
        record.sequence != state_.nextSequence - state_.count + offset) return false;
    Record frozen{};
    if (offset == 0 && store_.readFlight(frozen) && valid(frozen) &&
        frozen.sequence == record.sequence && frozen.boot == record.boot) record = frozen;
    return true;
  }
  bool prepare(Record& record, uint64_t boot, uint64_t nowMs, int64_t nowUnix) {
    if (!peek(record)) return false;
    Record frozen{};
    if (store_.readFlight(frozen) && valid(frozen) && frozen.sequence == record.sequence && frozen.boot == record.boot) {
      record = frozen; return true;
    }
    record = resolveTime(record, boot, nowMs, nowUnix);
    // This commit happens BEFORE any POST, so retries after restart keep identical JSON.
    if (!store_.writeFlight(record)) return fault();
    return true;
  }
  bool acknowledge(uint64_t sequence, bool rejected = false) {
    Record record{};
    if (!ready_) return false;
    if (!state_.count) return true;
    if (!peek(record)) return false;
    if (record.sequence != sequence) return true; // In-flight record was evicted by overflow.
    State next = state_;
    next.head = (next.head + 1) % Slots; --next.count;
    if (record.sensor == Sensor::Touch) next.pendingTouches -= record.count;
    if (rejected) ++next.rejected;
    return commit(next);
  }
  const State& state() const { return state_; }
  bool ready() const { return ready_; }
 private:
  bool fault() { ready_ = false; return false; }
  bool commit(State next) {
    next.generation = state_.generation + 1;
    seal(next);
    if (!store_.writeState(next.generation % 2, next)) return fault();
    state_ = next; return true;
  }
  Store& store_;
  State state_{};
  bool ready_ = false;
};
}
