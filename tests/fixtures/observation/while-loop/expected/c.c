#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t primer_count = 0;
    int64_t primer_sum = 0;
    while (primer_count < 4) {
        primer_sum = (primer_sum + primer_count);
        if (primer_count == 2) {
            bool primer_marker = true;
            printf("%s\n", (primer_marker) ? "true" : "false");
        }
        primer_count = (primer_count + 1);
    }
    printf("%lld\n", (long long)(primer_sum));
    return 0;
}
