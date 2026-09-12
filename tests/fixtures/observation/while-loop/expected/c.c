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
    int64_t primer_binding_0_count = 0;
    int64_t primer_binding_1_sum = 0;
    while (primer_binding_0_count < 4) {
        primer_binding_1_sum = primer_i64_add(primer_binding_1_sum, primer_binding_0_count, " node=9 bytes=67..78\n");
        if (primer_binding_0_count == 2) {
            bool primer_binding_2_marker = true;
            printf("%s\n", (primer_binding_2_marker) ? "true" : "false");
        }
        primer_binding_0_count = primer_i64_add(primer_binding_0_count, 1, " node=21 bytes=172..181\n");
    }
    printf("%lld\n", (long long)(primer_binding_1_sum));
    return 0;
}
