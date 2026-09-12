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

typedef struct primer_array_i64_2 {
    int64_t items[2];
} primer_array_i64_2;

static int64_t primer_array_get_i64_2(primer_array_i64_2 value, int64_t index, const char *origin) {
    if (index < 0 || index >= 2) {
        primer_runtime_fail("array-index-out-of-bounds", origin);
    }
    return value.items[index];
}

bool primer_fn_report_0(bool primer_binding_0_value);

bool primer_fn_report_0(bool primer_binding_0_value) {
    printf("%s\n", (primer_binding_0_value) ? "true" : "false");
    return primer_binding_0_value;
}

int main(void) {
    primer_array_i64_2 primer_binding_1_values = (primer_array_i64_2){ .items = { 4, 9 } };
    int64_t primer_binding_2_index = 2;
    printf("%s\n", (((primer_binding_2_index < 2) && (primer_array_get_i64_2(primer_binding_1_values, primer_binding_2_index, " node=16 bytes=134..147\n") > 0))) ? "true" : "false");
    printf("%s\n", (((primer_binding_2_index == 2) || primer_fn_report_0(false))) ? "true" : "false");
    printf("%s\n", ((false || (primer_fn_report_0(true) && ((primer_binding_2_index > 0) || primer_fn_report_0(false))))) ? "true" : "false");
    return 0;
}
