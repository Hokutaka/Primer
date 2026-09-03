#include <stdint.h>
#include <stdio.h>

int main(void) {
    int64_t primer_count = 40;
    primer_count = (primer_count + 2);
    float primer_ratio = 0.25f;
    primer_ratio = (primer_ratio * 2.0f);
    printf("%lld\n", (long long)(primer_count));
    printf("%.9g\n", (double)(primer_ratio));
    return 0;
}
