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

static int64_t primer_i64_neg(int64_t value, const char *origin) {
    if (value == INT64_MIN) {
        primer_runtime_fail("integer-overflow", origin);
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
        primer_binding_0_value = primer_i64_neg(1, " node=13 bytes=115..117\n");
    }
    printf("%lld\n", (long long)(primer_binding_0_value));
    return 0;
}
