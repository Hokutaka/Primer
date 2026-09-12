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

typedef struct primer_type_Row_0 {
    primer_array_i64_3 values;
} primer_type_Row_0;

int main(void) {
    primer_type_Row_0 primer_binding_0_first = (primer_type_Row_0){ .values = (primer_array_i64_3){ .items = { 1, 2, 3 } } };
    primer_type_Row_0 primer_binding_1_second = primer_binding_0_first;
    primer_binding_0_first = (primer_type_Row_0){ .values = (primer_array_i64_3){ .items = { 4, 5, 6 } } };
    printf("%lld\n", (long long)(primer_array_get_i64_3((primer_binding_1_second).values, 1, " node=15 bytes=145..161\n")));
    printf("%lld\n", (long long)(primer_array_get_i64_3((primer_binding_0_first).values, 2, " node=20 bytes=170..185\n")));
    return 0;
}
