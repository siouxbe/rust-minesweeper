#include <../target/rust_lib.hpp>

#include <iostream>

int main(const int, const char * []) {
    const CPerson p{"Abraham", 50};
    char buffer[128]{};
    if (!serialize_person_from_rust(&p, buffer, sizeof(buffer) - 1)) {
        std::cout << "Failed to serialize" << std::endl;
        return 1;
    }
    std::cout << buffer << std::endl;
    return 0;
}
