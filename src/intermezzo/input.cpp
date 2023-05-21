#include <cstdint>
#include <iostream>
#include <vector>

void copy_odd(const std::vector<int32_t> & src, std::vector<int32_t> & dst) {
    for (const int32_t n : src) {
        if (n & 0x1) {
            dst.push_back(n);
        }
    }
}

int main(const int, const char * []) {
    std::vector<int32_t> v{1, 2, 3, 4, 5};
    copy_odd(v, v);
    for (const int32_t n : v) {
        std::cout << n << std::endl;
    }
}
