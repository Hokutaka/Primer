#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    printf("%.9g\n", (double)(1e-20f));
    printf("%.17g\n", (double)(1e-20));
    printf("%s\n", ((1e-20 != 0.0)) ? "true" : "false");
    printf("%.9g\n", (double)(1e-45f));
    printf("%.17g\n", (double)(5e-324));
    printf("%.9g\n", (double)(3.4028234663852886e38f));
    printf("%.17g\n", (double)(1.7976931348623157e308));
    printf("%.9g\n", (double)((-0.0f)));
    printf("%.17g\n", (double)((-0.0)));
    printf("%.9g\n", (double)(0.0f));
    printf("%.17g\n", (double)(0.0));
    printf("%.9g\n", (double)(0.0001f));
    printf("%.17g\n", (double)(0.0001));
    printf("%.9g\n", (double)(1e9f));
    printf("%.17g\n", (double)(1e17));
    return 0;
}
