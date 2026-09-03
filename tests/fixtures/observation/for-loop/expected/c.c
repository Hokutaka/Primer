#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t primer_sum = 0;
    for (int64_t primer_i = 0; (primer_i < 6); primer_i = (primer_i + 1)) {
        if (primer_i < 2) {
            continue;
        }
        primer_sum = (primer_sum + primer_i);
    }
    printf("%lld\n", (long long)(primer_sum));
    return 0;
}
