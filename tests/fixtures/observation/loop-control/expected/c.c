#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t primer_value = 0;
    int64_t primer_sum = 0;
    while (primer_value < 10) {
        primer_value = (primer_value + 1);
        if (primer_value < 3) {
            continue;
        }
        if (primer_value > 5) {
            break;
        }
        primer_sum = (primer_sum + primer_value);
    }
    printf("%lld\n", (long long)(primer_sum));
    printf("%lld\n", (long long)(primer_value));
    return 0;
}
