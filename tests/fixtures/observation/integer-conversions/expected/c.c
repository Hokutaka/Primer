#include <stdint.h>
#include <stdio.h>

int64_t primer_fn_value_0(void);

int64_t primer_fn_value_0(void) {
    printf("%lld\n", (long long)(7));
    return 42;
}

int main(void) {
    int64_t primer_binding_0_compact = primer_fn_value_0();
    int64_t primer_binding_1_explicit = primer_binding_0_compact;
    printf("%lld\n", (long long)(primer_binding_1_explicit));
    return 0;
}
