#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t primer_value = 1;
    if (primer_value < 2) {
        primer_value = 42;
        bool primer_value = true;
        printf("%s\n", (primer_value) ? "true" : "false");
    } else {
        primer_value = (-1);
    }
    printf("%lld\n", (long long)(primer_value));
    return 0;
}
