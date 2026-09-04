#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

static int64_t primer_i64_sub(int64_t left, int64_t right) {
    if ((right < 0 && left > INT64_MAX + right) ||
        (right > 0 && left < INT64_MIN + right)) {
        primer_integer_overflow();
    }
    return left - right;
}

static int64_t primer_i64_mul(int64_t left, int64_t right) {
    if ((left > 0 && right > 0 && left > INT64_MAX / right) ||
        (left > 0 && right < 0 && right < INT64_MIN / left) ||
        (left < 0 && right > 0 && left < INT64_MIN / right) ||
        (left < 0 && right < 0 && left < INT64_MAX / right)) {
        primer_integer_overflow();
    }
    return left * right;
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

int main(void) {
    int64_t primer_value = 8;
    printf("%lld\n", (long long)(primer_i64_add(primer_value, 1)));
    printf("%lld\n", (long long)(primer_i64_sub(primer_value, 1)));
    printf("%lld\n", (long long)(primer_i64_mul(primer_value, 2)));
    printf("%lld\n", (long long)(primer_i64_div(primer_value, 2)));
    printf("%lld\n", (long long)(primer_i64_neg(primer_value)));
    return 0;
}
