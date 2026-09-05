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

typedef struct primer_array_array_i64_3_2 {
    primer_array_i64_3 items[2];
} primer_array_array_i64_3_2;

static primer_array_i64_3 primer_array_get_array_i64_3_2(primer_array_array_i64_3_2 value, int64_t index) {
    if (index < 0 || index >= 2) {
        fputs("primer: array index out of bounds\n", stderr);
        abort();
    }
    return value.items[index];
}

int main(void) {
    primer_array_array_i64_3_2 primer_binding_0_matrix = (primer_array_array_i64_3_2){ .items = { (primer_array_i64_3){ .items = { 1, 2, 3 } }, (primer_array_i64_3){ .items = { 4, 5, 6 } } } };
    primer_array_array_i64_3_2 primer_binding_1_copy = primer_binding_0_matrix;
    primer_binding_0_matrix = (primer_array_array_i64_3_2){ .items = { (primer_array_i64_3){ .items = { 7, 8, 9 } }, (primer_array_i64_3){ .items = { 10, 11, 12 } } } };
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_array_get_array_i64_3_2(primer_binding_1_copy, 1), 2)));
    printf("%lld\n", (long long)(primer_array_get_i64_3(primer_array_get_array_i64_3_2(primer_binding_0_matrix, 0), 1)));
    return 0;
}
