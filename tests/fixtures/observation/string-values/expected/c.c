#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stddef.h>
#include <string.h>
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif

typedef struct primer_string {
    const unsigned char *data;
    size_t length;
} primer_string;

static inline bool primer_string_equal(primer_string left, primer_string right) {
    return left.length == right.length &&
        (left.length == 0 || memcmp(left.data, right.data, left.length) == 0);
}

static inline void primer_print_string(primer_string value) {
    fwrite(value.data, 1, value.length, stdout);
    fputc('\n', stdout);
}

int main(void) {
#ifdef _WIN32
    if (_setmode(_fileno(stdout), _O_BINARY) == -1) {
        fputs("primer: cannot set stdout to binary mode\n", stderr);
        return 1;
    }
#endif
    primer_string primer_binding_0_text = (primer_string){ (const unsigned char *)"\346\227\245\346\234\254\350\252\236\012\000", 11 };
    primer_string primer_binding_1_saved = primer_binding_0_text;
    primer_binding_0_text = (primer_string){ (const unsigned char *)"\143\150\141\156\147\145\144", 7 };
    printf("%s\n", ((primer_string_equal(primer_binding_1_saved, (primer_string){ (const unsigned char *)"\346\227\245\346\234\254\350\252\236\012\000", 11 }))) ? "true" : "false");
    primer_print_string(primer_binding_0_text);
    return 0;
}
