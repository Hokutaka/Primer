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

static int64_t primer_i64_add(int64_t left, int64_t right, const char *origin) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        primer_runtime_fail("integer-overflow", origin);
    }
    return left + right;
}

int main(void) {
    int64_t primer_binding_0_value = 0;
    int64_t primer_binding_1_sum = 0;
    while (primer_binding_0_value < 10) {
        primer_binding_0_value = primer_i64_add(primer_binding_0_value, 1, " node=9 bytes=70..79\n");
        if (primer_binding_0_value < 3) {
            continue;
        }
        if (primer_binding_0_value > 5) {
            break;
        }
        primer_binding_1_sum = primer_i64_add(primer_binding_1_sum, primer_binding_0_value, " node=23 bytes=177..188\n");
    }
    printf("%lld\n", (long long)(primer_binding_1_sum));
    printf("%lld\n", (long long)(primer_binding_0_value));
    return 0;
}
