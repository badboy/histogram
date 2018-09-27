#include <iostream>
#include "histogram.h"

struct Hist {
  public:
    static Hist* FactoryGet(int min, int max, unsigned int bucket_count, const unsigned int* buckets);
    ~Hist();

    void Add(int value);
    std::string Serialize();
    std::string Persist();

  protected:
    Hist() : mPtr(nullptr) {}

    void *mPtr;
};

Hist* Hist::FactoryGet(int min, int max, unsigned int bucket_count, const unsigned int* buckets) {
  Hist *hist = new Hist;

  hist->mPtr = histogram_factory_get(min, max, bucket_count, buckets);

  return hist;
}

Hist::~Hist() {
  histogram_free(mPtr);
  mPtr = nullptr;
}

void Hist::Add(int value) {
  histogram_add(mPtr, value);
}

std::string Hist::Serialize() {
  char* s = histogram_serialize(mPtr);
  std::string str(s);
  histogram_free_cstr(s);

  return str;
}

std::string Hist::Persist() {
  char* s = histogram_serialize_persist(mPtr);
  std::string str(s);
  histogram_free_cstr(s);

  return str;
}

const unsigned int gHistogramBucketLowerBounds[] = {
0,1,2,INT_MAX,
0,1,2 ,INT_MAX,
0,1,2,3,4,5,6,7,8,9,10,11 ,INT_MAX,
0,1,2,4,7,13,24,44,80,146,267,487,889,1623,2962,5406,9867,18010,32872,60000 ,INT_MAX,
};

int main(void)
{
  size_t offset = 21;
  const unsigned int* buckets = &gHistogramBucketLowerBounds[offset];
  size_t count = 20;

  Hist* h = nullptr;
  h = Hist::FactoryGet(1, 60000, count, buckets);

  h->Add(20);

  std::string s = h->Serialize();

  std::cout << s << std::endl;

  delete h;

  return 0;
}
