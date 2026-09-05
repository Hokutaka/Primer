#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static int64_t primer_check_i8(int64_t value) {
    if (value < -128LL || value > 127LL) abort();
    return value;
}

static int64_t primer_check_u8(int64_t value) {
    if (value < 0LL || value > 255LL) abort();
    return value;
}

static int64_t primer_check_i16(int64_t value) {
    if (value < -32768LL || value > 32767LL) abort();
    return value;
}

static int64_t primer_check_u16(int64_t value) {
    if (value < 0LL || value > 65535LL) abort();
    return value;
}

static void primer_integer_overflow(void) {
    fputs("primer: integer operation produced a value outside the supported range\n", stderr);
    abort();
}

static int64_t primer_i64_add(int64_t left, int64_t right) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        primer_integer_overflow();
    }
    return left + right;
}

static int64_t primer_i64_div(int64_t left, int64_t right) {
    if (right == 0) {
        fputs("primer: cannot divide an integer by zero\n", stderr);
        abort();
    }
    if (left == INT64_MIN && right == -1) {
        primer_integer_overflow();
    }
    return left / right;
}

static int64_t primer_i64_neg(int64_t value) {
    if (value == INT64_MIN) {
        primer_integer_overflow();
    }
    return -value;
}

int64_t primer_fn_average_0(int64_t primer_left, int64_t primer_right);

int64_t primer_fn_average_0(int64_t primer_left, int64_t primer_right) {
    return primer_check_u8(primer_check_u16(primer_i64_div(primer_check_u16(primer_i64_add(primer_check_u16(primer_left), primer_check_u16(primer_right))), 2)));
}

int main(void) {
    int64_t primer_offset = primer_check_i8(primer_i64_neg(3));
    int64_t primer_reading = primer_check_i16(primer_i64_neg(32000));
    printf("%lld\n", (long long)(primer_check_i16(primer_i64_add(primer_reading, primer_check_i16(primer_offset)))));
    printf("%lld\n", (long long)(primer_fn_average_0(240, 80)));
    printf("%s\n", ((127 > -128)) ? "true" : "false");
    printf("%lld\n", (long long)(primer_check_u16(255)));
    return 0;
}
