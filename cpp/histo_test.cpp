#include <iostream>
#include "histogram.h"

class StaticHistogram final {
 public:
  ~StaticHistogram() {}
  static void operator delete(void* aHistogram) {
    histogram_free(reinterpret_cast<StaticHistogram*>(aHistogram));
  }

  static inline StaticHistogram* NewHistogram(
      int min, int max, size_t bucket_count, const unsigned int* buckets) {
    return histogram_factory_get(min, max, bucket_count, buckets);
  }

  inline void Add(unsigned int sample) { histogram_add(this, sample); }

  inline const char* Serialize() {
    return histogram_serialize(this);
  }

  inline const char* Persist() {
    return histogram_serialize_persist(this);
  }

 private:
  StaticHistogram() = delete;
  StaticHistogram(const StaticHistogram&) = delete;
  StaticHistogram& operator=(const StaticHistogram&) = delete;
};

const unsigned int gHistogramBucketLowerBounds[] = {
    0,    1,    2,       INT_MAX, 0,     1,       2,   INT_MAX, 0,
    1,    2,    3,       4,       5,     6,       7,   8,       9,
    10,   11,   INT_MAX, 0,       1,     2,       4,   7,       13,
    24,   44,   80,      146,     267,   487,     889, 1623,    2962,
    5406, 9867, 18010,   32872,   60000, INT_MAX,
};

int main(void) {
  size_t offset = 21;
  const unsigned int* buckets = &gHistogramBucketLowerBounds[offset];
  size_t count = 20;

  StaticHistogram* h = nullptr;
  h = StaticHistogram::NewHistogram(1, 60000, count, buckets);

  for (int i = 0; i < 10; i++) {
    h->Add(20 + i);
  }

   std::string s = h->Serialize();
   std::cout << "Serialized: " << s << std::endl;

   s = h->Persist();
   std::cout << "Persisted:  " << s << std::endl;

  delete h;

  return 0;
}
