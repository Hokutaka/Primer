#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void primer_integer_overflow(void) {
    fputs("primer: integer operation produced a value outside the supported range\n", stderr);
    abort();
}

static int64_t primer_i64_neg(int64_t value) {
    if (value == INT64_MIN) {
        primer_integer_overflow();
    }
    return -value;
}

int main(void) {
    int64_t primer_binding_0_value = 1;
    if (primer_binding_0_value < 2) {
        primer_binding_0_value = 42;
        bool primer_binding_1_value = true;
        printf("%s\n", (primer_binding_1_value) ? "true" : "false");
    } else {
        primer_binding_0_value = primer_i64_neg(1);
    }
    printf("%lld\n", (long long)(primer_binding_0_value));
    return 0;
}
