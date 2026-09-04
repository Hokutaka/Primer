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
    int64_t primer_sum = 0;
    for (int64_t primer_i = 0; (primer_i < 6); primer_i = primer_i64_add(primer_i, 1)) {
        if (primer_i < 2) {
            continue;
        }
        primer_sum = primer_i64_add(primer_sum, primer_i);
    }
    printf("%lld\n", (long long)(primer_sum));
    return 0;
}
