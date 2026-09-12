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
    bool primer_binding_0_truth = true;
    bool primer_binding_1_negated = (!primer_binding_0_truth);
    bool primer_binding_2_same = (primer_binding_0_truth == true);
    bool primer_binding_3_integer_order = (primer_i64_add(1, 2, " node=11 bytes=94..99\n") < 4);
    bool primer_binding_4_float_difference = (0.1f != 0.2f);
    printf("%s\n", (primer_binding_0_truth) ? "true" : "false");
    printf("%s\n", (primer_binding_1_negated) ? "true" : "false");
    printf("%s\n", (primer_binding_2_same) ? "true" : "false");
    printf("%s\n", (primer_binding_3_integer_order) ? "true" : "false");
    printf("%s\n", (primer_binding_4_float_difference) ? "true" : "false");
    return 0;
}
