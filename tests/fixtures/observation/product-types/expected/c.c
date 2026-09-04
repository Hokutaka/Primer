#include <stdint.h>
#include <stdio.h>

typedef struct primer_type_Point_0 {
    double x;
    double y;
} primer_type_Point_0;

typedef struct primer_type_Segment_1 {
    primer_type_Point_0 start;
    primer_type_Point_0 end;
} primer_type_Segment_1;

int main(void) {
    primer_type_Point_0 primer_current = (primer_type_Point_0){ .y = 2.0, .x = 0.0 };
    primer_type_Point_0 primer_saved = primer_current;
    primer_current = (primer_type_Point_0){ .x = 4.0, .y = 5.0 };
    primer_type_Segment_1 primer_segment = (primer_type_Segment_1){ .start = primer_saved, .end = primer_current };
    printf("%.17g\n", (double)((primer_saved).x));
    printf("%.17g\n", (double)((primer_saved).y));
    printf("%.17g\n", (double)(((primer_segment).start).y));
    printf("%.17g\n", (double)(((primer_segment).end).x));
    return 0;
}
