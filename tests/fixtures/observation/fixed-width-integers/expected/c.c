#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static int64_t primer_check_i32(int64_t value) {
    if (value < -2147483648LL || value > 2147483647LL) abort();
    return value;
}

static int64_t primer_check_u32(int64_t value) {
    if (value < 0LL || value > 4294967295LL) abort();
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

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right);

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right) {
    return primer_check_i32(primer_i64_add(primer_binding_0_left, primer_binding_1_right));
}

int main(void) {
    int64_t primer_binding_2_small = primer_fn_add_0(primer_check_i32(primer_i64_neg(3)), 5);
    int64_t primer_binding_3_large = 4294967295;
    printf("%lld\n", (long long)(primer_binding_2_small));
    printf("%lld\n", (long long)(primer_check_u32(primer_i64_div(primer_binding_3_large, 2))));
    printf("%lld\n", (long long)(primer_binding_3_large));
    printf("%s\n", ((primer_binding_3_large > 2147483648)) ? "true" : "false");
    printf("%lld\n", (long long)(primer_check_u32(primer_binding_2_small)));
    return 0;
}
