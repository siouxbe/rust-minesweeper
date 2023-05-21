#include <cassert>
#include <cstddef>
#include <iostream>
#include <vector>

class CachedVector {
public:
    CachedVector(size_t length) : data(length), last{} {}

    int * last_accessed() { return last; }

    int & operator[](const size_t index) {
        last = &data[index];
        return *last;
    }

private:
    std::vector<int> data;
    int * last;
};

void foobar() {
    CachedVector a(3);
    a[0] = 0; a[1] = 1; a[2] = 2;
    CachedVector b(a);
    assert(b.last_accessed() == nullptr);
}
