#include <stdbool.h>
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

static int64_t primer_check_i32(int64_t value, const char *code, const char *origin) {
    if (value < -2147483648LL || value > 2147483647LL) primer_runtime_fail(code, origin);
    return value;
}

static int64_t primer_check_u32(int64_t value, const char *code, const char *origin) {
    if (value < 0LL || value > 4294967295LL) primer_runtime_fail(code, origin);
    return value;
}

static int64_t primer_i64_add(int64_t left, int64_t right, const char *origin) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return left + right;
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

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right);

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right) {
    return primer_check_i32(primer_i64_add(primer_binding_0_left, primer_binding_1_right, " node=1 bytes=50..62\n"), "integer-overflow", " node=1 bytes=50..62\n");
}

int main(void) {
    int64_t primer_binding_2_small = primer_fn_add_0(primer_check_i32(primer_i64_neg(3, " node=6 bytes=83..85\n"), "integer-overflow", " node=6 bytes=83..85\n"), 5);
    int64_t primer_binding_3_large = 4294967295;
    printf("%lld\n", (long long)(primer_binding_2_small));
    printf("%lld\n", (long long)(primer_check_u32(primer_i64_div(primer_binding_3_large, 2, " node=14 bytes=136..145\n"), "division-overflow", " node=14 bytes=136..145\n")));
    printf("%lld\n", (long long)(primer_binding_3_large));
    printf("%s\n", ((primer_binding_3_large > 2147483648)) ? "true" : "false");
    printf("%lld\n", (long long)(primer_check_u32(primer_binding_2_small, "integer-conversion-out-of-range", " node=25 bytes=200..219\n")));
    return 0;
}
