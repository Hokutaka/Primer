#include <stdbool.h>
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

int main(void) {
    int64_t primer_binding_0_count = 0;
    int64_t primer_binding_1_sum = 0;
    while (primer_binding_0_count < 4) {
        primer_binding_1_sum = primer_i64_add(primer_binding_1_sum, primer_binding_0_count);
        if (primer_binding_0_count == 2) {
            bool primer_binding_2_marker = true;
            printf("%s\n", (primer_binding_2_marker) ? "true" : "false");
        }
        primer_binding_0_count = primer_i64_add(primer_binding_0_count, 1);
    }
    printf("%lld\n", (long long)(primer_binding_1_sum));
    return 0;
}
