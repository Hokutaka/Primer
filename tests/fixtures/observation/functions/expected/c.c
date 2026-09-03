#include <stdint.h>
#include <stdio.h>

int64_t primer_fn_add_0(int64_t primer_left, int64_t primer_right);
void primer_fn_show_1(int64_t primer_value);

int64_t primer_fn_add_0(int64_t primer_left, int64_t primer_right) {
    return (primer_left + primer_right);
}

void primer_fn_show_1(int64_t primer_value) {
    printf("%lld\n", (long long)(primer_value));
}

int main(void) {
    int64_t primer_answer = primer_fn_add_0(20, 22);
    primer_fn_show_1(primer_answer);
    return 0;
}
