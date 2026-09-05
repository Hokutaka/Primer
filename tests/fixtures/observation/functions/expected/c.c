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

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right);
void primer_fn_show_1(int64_t primer_binding_2_value);

int64_t primer_fn_add_0(int64_t primer_binding_0_left, int64_t primer_binding_1_right) {
    return primer_i64_add(primer_binding_0_left, primer_binding_1_right);
}

void primer_fn_show_1(int64_t primer_binding_2_value) {
    printf("%lld\n", (long long)(primer_binding_2_value));
}

int main(void) {
    int64_t primer_binding_3_answer = primer_fn_add_0(20, 22);
    primer_fn_show_1(primer_binding_3_answer);
    return 0;
}
