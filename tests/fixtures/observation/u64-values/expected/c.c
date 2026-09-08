#include <math.h>
#include <float.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static uint64_t primer_convert_i64_u64(int64_t value) {
    if (value < 0) { fputs("primer: integer conversion out of range\n", stderr); abort(); }
    return (uint64_t)value;
}

static uint64_t primer_u64_div(uint64_t left, uint64_t right) {
    if (right == 0) { fputs("primer: integer division by zero\n", stderr); abort(); }
    return left / right;
}

static uint64_t primer_u64_shr(uint64_t left, uint64_t right) {
    if (right >= 64) { fputs("primer: u64 invalid shift count\n", stderr); abort(); }
    return left >> right;
}

static void primer_integer_overflow(void) {
    fputs("primer: integer operation produced a value outside the supported range\n", stderr);
    abort();
}

int main(void) {
    int64_t _primer_bit_left_9, _primer_bit_right_9;
    int64_t _primer_bit_left_13, _primer_bit_right_13;
    uint64_t primer_binding_0_maximum = UINT64_C(18446744073709551615);
    printf("%llu\n", (unsigned long long)(primer_binding_0_maximum));
    printf("%s\n", ((primer_binding_0_maximum > UINT64_C(0))) ? "true" : "false");
    printf("%llu\n", (unsigned long long)((_primer_bit_left_9 = primer_binding_0_maximum, _primer_bit_right_9 = UINT64_C(2), primer_u64_div(_primer_bit_left_9, _primer_bit_right_9))));
    printf("%llu\n", (unsigned long long)((_primer_bit_left_13 = primer_binding_0_maximum, _primer_bit_right_13 = UINT64_C(63), primer_u64_shr(_primer_bit_left_13, _primer_bit_right_13))));
    printf("%llu\n", (unsigned long long)(primer_convert_i64_u64(42)));
    return 0;
}
