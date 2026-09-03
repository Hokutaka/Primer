#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    bool primer_truth = true;
    bool primer_negated = (!primer_truth);
    bool primer_same = (primer_truth == true);
    bool primer_integer_order = ((1 + 2) < 4);
    bool primer_float_difference = (0.1f != 0.2f);
    printf("%s\n", (primer_truth) ? "true" : "false");
    printf("%s\n", (primer_negated) ? "true" : "false");
    printf("%s\n", (primer_same) ? "true" : "false");
    printf("%s\n", (primer_integer_order) ? "true" : "false");
    printf("%s\n", (primer_float_difference) ? "true" : "false");
    return 0;
}
