#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct primer_array_i64_3 {
    int64_t items[3];
} primer_array_i64_3;

static int64_t primer_array_get_i64_3(primer_array_i64_3 value, int64_t index) {
    if (index < 0 || index >= 3) {
        fputs("primer: array index out of bounds\n", stderr);
        abort();
    }
    return value.items[index];
}

int main(void) {
    primer_array_i64_3 primer_values = (primer_array_i64_3){ .items = { 2, 4, 6 } };
    primer_array_i64_3 primer_copy = primer_values;
    primer_values = (primer_array_i64_3){ .items = { 1, 3, 5 } };
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_copy, 2)));
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_values, 1)));
    return 0;
}
