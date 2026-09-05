#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct primer_type_Point_0 {
    int64_t x;
    int64_t y;
} primer_type_Point_0;

typedef struct primer_array_type_Point_0_2 {
    primer_type_Point_0 items[2];
} primer_array_type_Point_0_2;

static primer_type_Point_0 primer_array_get_type_Point_0_2(primer_array_type_Point_0_2 value, int64_t index) {
    if (index < 0 || index >= 2) {
        fputs("primer: array index out of bounds\n", stderr);
        abort();
    }
    return value.items[index];
}

int main(void) {
    primer_array_type_Point_0_2 primer_binding_0_points = (primer_array_type_Point_0_2){ .items = { (primer_type_Point_0){ .x = 1, .y = 2 }, (primer_type_Point_0){ .x = 3, .y = 4 } } };
    primer_array_type_Point_0_2 primer_binding_1_copy = primer_binding_0_points;
    primer_binding_0_points = (primer_array_type_Point_0_2){ .items = { (primer_type_Point_0){ .x = 5, .y = 6 }, (primer_type_Point_0){ .x = 7, .y = 8 } } };
    printf("%lld\n", (long long)((primer_array_get_type_Point_0_2(primer_binding_1_copy, 1)).x));
    printf("%lld\n", (long long)((primer_array_get_type_Point_0_2(primer_binding_0_points, 0)).y));
    return 0;
}
