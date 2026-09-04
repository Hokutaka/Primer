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
    int64_t primer_count = 40;
    primer_count = primer_i64_add(primer_count, 2);
    float primer_ratio = 0.25f;
    primer_ratio = (primer_ratio * 2.0f);
    printf("%lld\n", (long long)(primer_count));
    printf("%.9g\n", (double)(primer_ratio));
    return 0;
}
