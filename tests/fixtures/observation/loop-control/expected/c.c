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
    int64_t primer_binding_0_value = 0;
    int64_t primer_binding_1_sum = 0;
    while (primer_binding_0_value < 10) {
        primer_binding_0_value = primer_i64_add(primer_binding_0_value, 1);
        if (primer_binding_0_value < 3) {
            continue;
        }
        if (primer_binding_0_value > 5) {
            break;
        }
        primer_binding_1_sum = primer_i64_add(primer_binding_1_sum, primer_binding_0_value);
    }
    printf("%lld\n", (long long)(primer_binding_1_sum));
    printf("%lld\n", (long long)(primer_binding_0_value));
    return 0;
}
