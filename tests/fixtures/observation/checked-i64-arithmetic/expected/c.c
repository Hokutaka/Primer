#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void primer_runtime_fail(const char *code, const char *origin) {
    fflush(stdout);
    fputs("primer: runtime-v1 code=", stderr);
    fputs(code, stderr);
    fputs(origin, stderr);
    fflush(stderr);
    abort();
}

static int64_t primer_i64_add(int64_t left, int64_t right, const char *origin) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return left + right;
}

static int64_t primer_i64_sub(int64_t left, int64_t right, const char *origin) {
    if ((right < 0 && left > INT64_MAX + right) ||
        (right > 0 && left < INT64_MIN + right)) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return left - right;
}

static int64_t primer_i64_mul(int64_t left, int64_t right, const char *origin) {
    if ((left > 0 && right > 0 && left > INT64_MAX / right) ||
        (left > 0 && right < 0 && right < INT64_MIN / left) ||
        (left < 0 && right > 0 && left < INT64_MIN / right) ||
        (left < 0 && right < 0 && left < INT64_MAX / right)) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return left * right;
}

static int64_t primer_i64_div(int64_t left, int64_t right, const char *origin) {
    if (right == 0) {
        primer_runtime_fail("division-by-zero", origin);
    }
    if (left == INT64_MIN && right == -1) {
        primer_runtime_fail("division-overflow", origin);
    }
    return left / right;
}

static int64_t primer_i64_neg(int64_t value, const char *origin) {
    if (value == INT64_MIN) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return -value;
}

int main(void) {
    int64_t primer_binding_0_value = 8;
    printf("%lld\n", (long long)(primer_i64_add(primer_binding_0_value, 1, " node=3 bytes=22..31\n")));
    printf("%lld\n", (long long)(primer_i64_sub(primer_binding_0_value, 1, " node=7 bytes=40..49\n")));
    printf("%lld\n", (long long)(primer_i64_mul(primer_binding_0_value, 2, " node=11 bytes=58..67\n")));
    printf("%lld\n", (long long)(primer_i64_div(primer_binding_0_value, 2, " node=15 bytes=76..85\n")));
    printf("%lld\n", (long long)(primer_i64_neg(primer_binding_0_value, " node=19 bytes=94..100\n")));
    return 0;
}
