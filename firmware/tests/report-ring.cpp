#include "../shared/ReportStore.h"
#include <array>
#include <cassert>
#include <filesystem>
#include <iostream>
using namespace offline;
struct MemoryStore {
  std::array<Record, Slots> slots{};
  std::array<State, 2> states{};
  Record flight{};
  bool fresh = true, failState = false;
  MemoryStore() { for (auto& s : states) s.magic = 0; }
  bool pristine() { bool result = fresh; fresh = false; return result; }
  bool readState(unsigned i, State& s) { s = states[i]; return true; }
  bool writeState(unsigned i, const State& s) { if (failState) return false; states[i] = s; return true; }
  bool readSlot(unsigned i, Record& r) { r = slots[i]; return true; }
  bool writeSlot(unsigned i, const Record& r) { slots[i] = r; return true; }
  bool readFlight(Record& r) { r = flight; return valid(r); }
  bool writeFlight(const Record& r) { flight = r; return true; }
};
Record reading() { Record r; r.boot = 42; r.count = 3; r.observedMs = 1000; return r; }
int main() {
  MemoryStore store; Ring<MemoryStore> ring(store); assert(ring.begin(7));
  for (unsigned i = 0; i < Capacity + 100; ++i) assert(ring.append(reading()));
  assert(ring.state().count == Capacity && ring.state().dropped == 100);
  assert(ring.state().pendingTouches == Capacity * 3);
  Record r; assert(ring.peek(r) && r.sequence == 101);
  State beforeInspection = ring.state();
  assert(ring.inspect(0, r) && r.sequence == 101);
  assert(ring.inspect(Capacity - 1, r) && r.sequence == Capacity + 100);
  assert(!ring.inspect(Capacity, r));
  assert(memcmp(&beforeInspection, &ring.state(), sizeof(State)) == 0);
  assert(!valid(store.flight)); // Viewing does not create an in-flight snapshot.
  assert(ring.acknowledge(1) && ring.state().count == Capacity);
  // Failed metadata commit leaves the previous full queue intact, including its oldest slot.
  store.failState = true; assert(!ring.append(reading())); store.failState = false;
  Ring<MemoryStore> recovered(store); assert(recovered.begin(7));
  assert(recovered.peek(r) && r.sequence == 101);
  assert(recovered.prepare(r, 42, 11000, 1791500010));
  assert(r.quality == TimeQuality::Estimated && r.observedUnix == 1791500000);
  assert(recovered.inspect(0, r) && r.quality == TimeQuality::Estimated && r.observedUnix == 1791500000);
  assert(recovered.inspect(1, r) && r.quality == TimeQuality::Pending);
  Ring<MemoryStore> reboot(store); assert(reboot.begin(7));
  assert(reboot.prepare(r, 99, 5000, 1791600000));
  assert(r.quality == TimeQuality::Estimated && r.observedUnix == 1791500000);
  assert(reboot.acknowledge(r.sequence));
  assert(reboot.prepare(r, 99, 5000, 1791600000));
  assert(r.quality == TimeQuality::Unknown && r.observedUnix == 0);
  assert(reboot.acknowledge(r.sequence, true)); assert(reboot.state().rejected == 1);
  for (unsigned i = 103; i <= Capacity + 100; ++i) {
    assert(reboot.peek(r) && r.sequence == i); assert(reboot.acknowledge(i));
  }
  assert(reboot.state().count == 0 && reboot.state().pendingTouches == 0);
  Ring<MemoryStore> wrongTarget(store); assert(!wrongTarget.begin(8));
  assert(reboot.append(reading()));
  store.slots[reboot.state().head].count ^= 1;
  Ring<MemoryStore> corrupt(store); assert(!corrupt.begin(7));
  // Actual segmented files, metadata replacement, and reopening after wraparound.
  char directory[] = "/tmp/sensor-ring-test-XXXXXX"; assert(mkdtemp(directory));
  auto path = std::string(directory) + "/queue";
  FileStore files(path.c_str()); Ring<FileStore> disk(files); assert(disk.begin(9));
  for (unsigned i = 0; i < Capacity + 5; ++i) assert(disk.append(reading()));
  Ring<FileStore> reopened(files); assert(reopened.begin(9));
  assert(reopened.state().dropped == 5 && reopened.peek(r) && r.sequence == 6);
  assert(reopened.prepare(r, 42, 2000, 1791500000));
  Ring<FileStore> retry(files); assert(retry.begin(9));
  Record same; assert(retry.prepare(same, 99, 5000, 1791600000));
  assert(memcmp(&r, &same, sizeof(r)) == 0);
  std::filesystem::remove_all(directory);
  std::cout << "Ring FIFO, overflow, interrupted commit, restart, timestamps, corruption and file storage passed\n";
}
