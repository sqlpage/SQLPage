// Finite tube radiance added to Three's physical lighting declarations.
uniform vec4 uJewelFirst[JEWEL_EDGE_COUNT];
uniform vec3 uJewelSecond[JEWEL_EDGE_COUNT];
uniform vec3 uJewelRadiance;

vec3 jewelTubeLight(
  vec3 lightStart, vec3 lightEnd, float sourceRadius,
  vec3 point, vec3 N, vec3 V, vec3 R,
  PhysicalMaterial surface, float alpha
) {
  vec3 L0 = lightStart - point;
  vec3 L1 = lightEnd - point;
  vec3 Ld = L1 - L0;
  float len2 = dot(Ld, Ld);
  float len = sqrt(max(len2, 1e-10));
  float d0 = max(length(L0), 1e-5);
  float d1 = max(length(L1), 1e-5);
  vec3 direction0 = L0 / d0;
  vec3 direction1 = L1 / d1;
  float NoL0 = dot(N, direction0);
  float NoL1 = dot(N, direction1);
  if (max(NoL0, NoL1) <= 0.0) return vec3(0.0);

  // Closest point of the finite source centreline to the reflection ray.
  // Unlike an image on a disk, this follows the camera and the 3D source.
  float rd = dot(R, Ld);
  float r0 = dot(R, L0);
  float denominator = max(len2 - rd * rd, len2 * 1e-6);
  float t = clamp((r0 * rd - dot(L0, Ld)) / denominator, 0.0, 1.0);
  vec3 representative = L0 + t * Ld;
  if (dot(R, representative) < 0.0) {
    // Equations treating R as an infinite line have a discontinuity when
    // the emitter lies behind the ray. Explicit endpoint selection fixes it.
    representative = dot(R, L0) / d0 > dot(R, L1) / d1 ? L0 : L1;
  }

  // Move from the centreline toward the finite cylindrical surface.
  vec3 toRay = max(dot(representative, R), 0.0) * R - representative;
  float rayDistance = max(length(toRay), 1e-6);
  representative += toRay * min(sourceRadius / rayDistance, 1.0);
  float distance = max(length(representative), sourceRadius + 1e-5);
  vec3 L = representative / distance;
  if (dot(N, L) <= 0.0) return vec3(0.0);

  vec3 mid = .5 * (L0 + L1);
  float midDistance = max(length(mid), sourceRadius + 1e-5);
  vec3 midDirection = mid / midDistance;
  vec3 axis = Ld / len;
  float projectedExtent = sqrt(max(1.0 - pow(dot(axis, midDirection), 2.0), 0.0));

  // Integral of N.L / |L|^3 along the finite segment, times its projected
  // tube width. The length factor preserves radiance as geometry scales.
  float lineIrradiance = max(NoL0 + NoL1, 0.0) * len /
    max(d0 * d1 + dot(L0, L1), 1e-6);
  float irradiance = lineIrradiance * (2.0 * sourceRadius * projectedExtent);
  irradiance += PI * sourceRadius * sourceRadius /
    (midDistance * midDistance) * max(dot(N, midDirection), 0.0);

  // Normalize in half-vector space, where the microfacet GGX distribution
  // is defined. This includes stretching at grazing angles instead of
  // retaining the energy normalization of a head-on, isotropic mirror.
  float halfSpan = length(normalize(direction0 + V) - normalize(direction1 + V));
  float alphaLine = min(1.0, alpha + halfSpan * (2.0 / 3.0));
  float alphaSphere = min(1.0, alpha + sourceRadius / (3.0 * distance));
  float alphaSphereAcross = min(1.0, alpha + sourceRadius /
    (3.0 * distance * max(dot(N, V), .08)));
  float normalization = (alpha / max(alphaLine, 1e-5)) *
    alpha * alpha / max(alphaSphere * alphaSphereAcross, 1e-10);
  return BRDF_GGX(L, V, N, surface) * irradiance * normalization;
}
