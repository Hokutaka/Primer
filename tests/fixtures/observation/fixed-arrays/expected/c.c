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

typedef struct primer_array_i64_3 {
    int64_t items[3];
} primer_array_i64_3;

static int64_t primer_array_get_i64_3(primer_array_i64_3 value, int64_t index, const char *origin) {
    if (index < 0 || index >= 3) {
        primer_runtime_fail("array-index-out-of-bounds", origin);
    }
    return value.items[index];
}

int main(void) {
    primer_array_i64_3 primer_binding_0_values = (primer_array_i64_3){ .items = { 2, 4, 6 } };
    primer_array_i64_3 primer_binding_1_copy = primer_binding_0_values;
    primer_binding_0_values = (primer_array_i64_3){ .items = { 1, 3, 5 } };
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_binding_1_copy, 2, " node=13 bytes=85..92\n")));
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_binding_0_values, 1, " node=17 bytes=101..110\n")));
    return 0;
}
