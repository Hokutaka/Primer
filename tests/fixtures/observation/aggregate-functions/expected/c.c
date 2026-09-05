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

typedef struct primer_array_i64_2 {
    int64_t items[2];
} primer_array_i64_2;

static int64_t primer_array_get_i64_2(primer_array_i64_2 value, int64_t index) {
    if (index < 0 || index >= 2) {
        fputs("primer: array index out of bounds\n", stderr);
        abort();
    }
    return value.items[index];
}

typedef struct primer_array_array_i64_2_2 {
    primer_array_i64_2 items[2];
} primer_array_array_i64_2_2;

static primer_array_i64_2 primer_array_get_array_i64_2_2(primer_array_array_i64_2_2 value, int64_t index) {
    if (index < 0 || index >= 2) {
        fputs("primer: array index out of bounds\n", stderr);
        abort();
    }
    return value.items[index];
}

typedef struct primer_type_Point_0 {
    int64_t x;
    int64_t y;
} primer_type_Point_0;

primer_type_Point_0 primer_fn_move_x_0(primer_type_Point_0 primer_binding_0_point, int64_t primer_binding_1_amount);
primer_type_Point_0 primer_fn_move_twice_1(primer_type_Point_0 primer_binding_2_point, int64_t primer_binding_3_amount);
primer_array_i64_2 primer_fn_first_row_2(primer_array_array_i64_2_2 primer_binding_4_matrix);
primer_array_array_i64_2_2 primer_fn_duplicate_3(primer_array_i64_2 primer_binding_5_row);
primer_array_array_i64_2_2 primer_fn_duplicate_first_row_4(primer_array_array_i64_2_2 primer_binding_6_matrix);

primer_type_Point_0 primer_fn_move_x_0(primer_type_Point_0 primer_binding_0_point, int64_t primer_binding_1_amount) {
    return (primer_type_Point_0){ .x = primer_i64_add((primer_binding_0_point).x, primer_binding_1_amount), .y = (primer_binding_0_point).y };
}

primer_type_Point_0 primer_fn_move_twice_1(primer_type_Point_0 primer_binding_2_point, int64_t primer_binding_3_amount) {
    return primer_fn_move_x_0(primer_fn_move_x_0(primer_binding_2_point, primer_binding_3_amount), primer_binding_3_amount);
}

primer_array_i64_2 primer_fn_first_row_2(primer_array_array_i64_2_2 primer_binding_4_matrix) {
    return primer_array_get_array_i64_2_2(primer_binding_4_matrix, 0);
}

primer_array_array_i64_2_2 primer_fn_duplicate_3(primer_array_i64_2 primer_binding_5_row) {
    return (primer_array_array_i64_2_2){ .items = { primer_binding_5_row, primer_binding_5_row } };
}

primer_array_array_i64_2_2 primer_fn_duplicate_first_row_4(primer_array_array_i64_2_2 primer_binding_6_matrix) {
    return primer_fn_duplicate_3(primer_fn_first_row_2(primer_binding_6_matrix));
}

int main(void) {
    primer_type_Point_0 primer_binding_7_original = (primer_type_Point_0){ .x = 2, .y = 3 };
    primer_type_Point_0 primer_binding_8_moved = primer_fn_move_twice_1(primer_binding_7_original, 5);
    primer_array_array_i64_2_2 primer_binding_9_matrix = (primer_array_array_i64_2_2){ .items = { (primer_array_i64_2){ .items = { 1, 2 } }, (primer_array_i64_2){ .items = { 3, 4 } } } };
    primer_array_array_i64_2_2 primer_binding_10_rows = primer_fn_duplicate_first_row_4(primer_binding_9_matrix);
    printf("%lld\n", (long long)((primer_binding_7_original).x));
    printf("%lld\n", (long long)((primer_binding_8_moved).x));
    printf("%lld\n", (long long)((primer_binding_8_moved).y));
    printf("%lld\n", (long long)(primer_array_get_i64_2(primer_array_get_array_i64_2_2(primer_binding_9_matrix, 1), 0)));
    printf("%lld\n", (long long)(primer_array_get_i64_2(primer_array_get_array_i64_2_2(primer_binding_10_rows, 0), 1)));
    printf("%lld\n", (long long)(primer_array_get_i64_2(primer_array_get_array_i64_2_2(primer_binding_10_rows, 1), 0)));
    return 0;
}
