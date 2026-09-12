#include <math.h>
#include <float.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static void primer_runtime_fail(const char *code, const char *origin) {
    fflush(stdout);
    fputs("primer: runtime-v1 code=", stderr);
    fputs(code, stderr);
    fputs(origin, stderr);
    fflush(stderr);
    abort();
}

static double primer_convert_i16_f64(int64_t value, const char *origin) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) primer_runtime_fail("conversion-inexact", origin);
    if ((int64_t)number != value) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static float primer_convert_u32_f32(int64_t value, const char *origin) {
    float result = (float)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) primer_runtime_fail("conversion-inexact", origin);
    if ((int64_t)number != value) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static double primer_convert_u32_f64(int64_t value, const char *origin) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) primer_runtime_fail("conversion-inexact", origin);
    if ((int64_t)number != value) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static double primer_convert_i64_f64(int64_t value, const char *origin) {
    double result = (double)value;
    double number = (double)result;
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) primer_runtime_fail("conversion-inexact", origin);
    if ((int64_t)number != value) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static int64_t primer_convert_f32_i16(float value, const char *origin) {
    double number = (double)value;
    if (!isfinite(number)) primer_runtime_fail("conversion-not-finite", origin);
    if (number == 0.0 && signbit(number)) primer_runtime_fail("conversion-negative-zero", origin);
    if (number < -32768.0 || number >= 32768.0) primer_runtime_fail("conversion-out-of-range", origin);
    int64_t result = (int64_t)number;
    if ((double)result != number) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static double primer_convert_f32_f64(float value, const char *origin) {
    if (isnan(value)) primer_runtime_fail("conversion-nan", origin);
    return (double)value;
}

static int64_t primer_convert_f64_i64(double value, const char *origin) {
    double number = (double)value;
    if (!isfinite(number)) primer_runtime_fail("conversion-not-finite", origin);
    if (number == 0.0 && signbit(number)) primer_runtime_fail("conversion-negative-zero", origin);
    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) primer_runtime_fail("conversion-out-of-range", origin);
    int64_t result = (int64_t)number;
    if ((double)result != number) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

static float primer_convert_f64_f32(double value, const char *origin) {
    if (isnan(value)) primer_runtime_fail("conversion-nan", origin);
    if (isinf(value)) return signbit(value) ? -INFINITY : INFINITY;
    if (value > (double)FLT_MAX || value < -(double)FLT_MAX) primer_runtime_fail("conversion-out-of-range", origin);
    float result = (float)value;
    if ((double)result != value) primer_runtime_fail("conversion-inexact", origin);
    return result;
}

double primer_fn_measure_0(int64_t primer_binding_0_value);

double primer_fn_measure_0(int64_t primer_binding_0_value) {
    double _primer_eval_0;
    double _primer_eval_1;
    return (_primer_eval_0 = primer_convert_i16_f64(primer_binding_0_value, " node=2 bytes=43..53\n"), _primer_eval_1 = primer_convert_i64_f64(2, " node=4 bytes=56..62\n"), (_primer_eval_0 / _primer_eval_1));
}

int main(void) {
    int64_t primer_binding_1_count = 42;
    double primer_binding_2_wide = primer_convert_u32_f64(primer_binding_1_count, " node=9 bytes=95..114\n");
    float primer_binding_3_narrow = primer_convert_f64_f32(primer_binding_2_wide, " node=12 bytes=130..139\n");
    printf("%lld\n", (long long)(primer_convert_f32_i16(primer_binding_3_narrow, " node=15 bytes=147..158\n")));
    printf("%lld\n", (long long)(primer_convert_f64_i64(primer_binding_2_wide, " node=18 bytes=167..176\n")));
    printf("%.17g\n", (double)(primer_convert_f32_f64(primer_binding_3_narrow, " node=21 bytes=185..196\n")));
    printf("%.9g\n", (double)(primer_convert_u32_f32(primer_binding_1_count, " node=24 bytes=205..215\n")));
    printf("%.17g\n", (double)(primer_fn_measure_0(3)));
    printf("%.9g\n", (double)(primer_convert_f64_f32((-0.0), " node=30 bytes=243..252\n")));
    return 0;
}
