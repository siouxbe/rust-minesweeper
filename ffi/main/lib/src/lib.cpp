#include "../target/rust_lib.hpp"

#include <cstdio>

extern "C" {
    
size_t serialize_person_from_c(
    const CPerson * const p,
    char * const buffer,
    const size_t buffer_len) {
    return snprintf(buffer, buffer_len, "Person %s is %ud years of age", p->name, p->age);
}

}
