#include <math.h>
#include <float.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static double primer_convert_i16_f64(int64_t value) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();
    if ((int64_t)number != value) abort();
    return result;
}

static float primer_convert_u32_f32(int64_t value) {
    float result = (float)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();
    if ((int64_t)number != value) abort();
    return result;
}

static double primer_convert_u32_f64(int64_t value) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();
    if ((int64_t)number != value) abort();
    return result;
}

static double primer_convert_i64_f64(int64_t value) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();
    if ((int64_t)number != value) abort();
    return result;
}

static int64_t primer_convert_f32_i16(float value) {
    double number = (double)value;
    if (!isfinite(number)) abort();
    if (number == 0.0 && signbit(number)) abort();
    if (number < -32768.0 || number >= 32768.0) abort();
    int64_t result = (int64_t)number;
    if ((double)result != number) abort();
    return result;
}

static double primer_convert_f32_f64(float value) {
    if (isnan(value)) abort();
    return (double)value;
}

static int64_t primer_convert_f64_i64(double value) {
    double number = (double)value;
    if (!isfinite(number)) abort();
    if (number == 0.0 && signbit(number)) abort();
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();
    int64_t result = (int64_t)number;
    if ((double)result != number) abort();
    return result;
}

static float primer_convert_f64_f32(double value) {
    if (isnan(value)) abort();
    if (isinf(value)) return signbit(value) ? -INFINITY : INFINITY;
    if (value > (double)FLT_MAX || value < -(double)FLT_MAX) abort();
    float result = (float)value;
    if ((double)result != value) abort();
    return result;
}

static void primer_integer_overflow(void) {
    fputs("primer: integer operation produced a value outside the supported range\n", stderr);
    abort();
}

double primer_fn_measure_0(int64_t primer_binding_0_value);

double primer_fn_measure_0(int64_t primer_binding_0_value) {
    double _primer_eval_0;
    double _primer_eval_1;
    return (_primer_eval_0 = primer_convert_i16_f64(primer_binding_0_value), _primer_eval_1 = primer_convert_i64_f64(2), (_primer_eval_0 / _primer_eval_1));
}

int main(void) {
    int64_t primer_binding_1_count = 42;
    double primer_binding_2_wide = primer_convert_u32_f64(primer_binding_1_count);
    float primer_binding_3_narrow = primer_convert_f64_f32(primer_binding_2_wide);
    printf("%lld\n", (long long)(primer_convert_f32_i16(primer_binding_3_narrow)));
    printf("%lld\n", (long long)(primer_convert_f64_i64(primer_binding_2_wide)));
    printf("%.17g\n", (double)(primer_convert_f32_f64(primer_binding_3_narrow)));
    printf("%.9g\n", (double)(primer_convert_u32_f32(primer_binding_1_count)));
    printf("%.17g\n", (double)(primer_fn_measure_0(3)));
    printf("%.9g\n", (double)(primer_convert_f64_f32((-0.0))));
    return 0;
}
