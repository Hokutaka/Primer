#include <stdint.h>
#include <stdio.h>

int main(void) {
    float primer_single = (0.1f + 0.2f);
    double primer_double = (0.1 + 0.2);
    double primer_inferred = (0.1 + 0.2);
    float primer_suffixed = (0.1f + 0.2f);
    printf("%.9g\n", (double)(primer_single));
    printf("%.17g\n", (double)(primer_double));
    printf("%.17g\n", (double)(primer_inferred));
    printf("%.9g\n", (double)(primer_suffixed));
    return 0;
}
