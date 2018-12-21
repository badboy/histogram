/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_telemetry_histogram_h__
#define mozilla_telemetry_histogram_h__

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <cstdint>
#include <cstdlib>

struct Snapshot;

// A histogram created from static data for ranges.
struct StaticHistogram;

extern "C" {

// Add a single value to the given histogram.
void histogram_add(StaticHistogram *histogram, unsigned int sample);

// Get the number of buckets in this histogram.
uintptr_t histogram_bucket_count(const StaticHistogram *histogram);

// Clear the stored data in the histogram
void histogram_clear(StaticHistogram *histogram);

// Create a new histogram from an external array of ranges.
StaticHistogram *histogram_factory_get(unsigned int min,
                                       unsigned int max,
                                       uintptr_t bucket_count,
                                       const int *ranges);

// Free a histogram's memory.
void histogram_free(StaticHistogram *histogram);

// Deallocate a null-terminated string.
void histogram_free_cstr(char *s);

// Check if this histogram recorded any values.
bool histogram_is_empty(const StaticHistogram *histogram);

uint32_t histogram_ranges(const StaticHistogram *histogram, int idx);

// Serialize the histogram into a packed representation.
//
// The returned data is null-terminated. It should be passed back to `histogram_free_cstr` to
// deallocate after usage.
char *histogram_serialize(StaticHistogram *histogram);

// Serialize the histogram into a persistable JSON string.
//
// The returned data is null-terminated. It should be passed back to `histogram_free_cstr` to
// deallocate after usage.
char *histogram_serialize_persist(StaticHistogram *histogram);

Snapshot *histogram_snapshot(const StaticHistogram *histogram);

uint32_t histogram_snapshot_counts(const Snapshot *snapshot, int idx);

void histogram_snapshot_free(Snapshot *snapshot);

uint32_t histogram_snapshot_sum(const Snapshot *snapshot);

} // extern "C"

#endif // mozilla_telemetry_histogram_h__
