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
    bool primer_truth = true;
    bool primer_negated = (!primer_truth);
    bool primer_same = (primer_truth == true);
    bool primer_integer_order = (primer_i64_add(1, 2) < 4);
    bool primer_float_difference = (0.1f != 0.2f);
    printf("%s\n", (primer_truth) ? "true" : "false");
    printf("%s\n", (primer_negated) ? "true" : "false");
    printf("%s\n", (primer_same) ? "true" : "false");
    printf("%s\n", (primer_integer_order) ? "true" : "false");
    printf("%s\n", (primer_float_difference) ? "true" : "false");
    return 0;
}
