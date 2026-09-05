#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

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

bool primer_fn_report_0(bool primer_value);

bool primer_fn_report_0(bool primer_value) {
    printf("%s\n", (primer_value) ? "true" : "false");
    return primer_value;
}

int main(void) {
    primer_array_i64_2 primer_values = (primer_array_i64_2){ .items = { 4, 9 } };
    int64_t primer_index = 2;
    printf("%s\n", (((primer_index < 2) && (primer_array_get_i64_2(primer_values, primer_index) > 0))) ? "true" : "false");
    printf("%s\n", (((primer_index == 2) || primer_fn_report_0(false))) ? "true" : "false");
    printf("%s\n", ((false || (primer_fn_report_0(true) && ((primer_index > 0) || primer_fn_report_0(false))))) ? "true" : "false");
    return 0;
}
