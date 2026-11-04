/* Local Peggle fork: native host presentation; original 3:2 guest raster. */
#ifndef PEGGLE_OUTPUT_POLICY_H
#define PEGGLE_OUTPUT_POLICY_H
#include <math.h>
static inline double peggle_output_scale(double width,double height,double fallback) {
    if (!isfinite(fallback)||fallback<=0) return 1.0;
    return fallback;
}
#endif
