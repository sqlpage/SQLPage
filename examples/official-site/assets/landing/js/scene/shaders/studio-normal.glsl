// A tiny surface relief scatters the grazing studio light. Derivatives filter
// the same grain used for roughness, avoiding a second texture or noise field.
#include <normal_fragment_maps>
float floorHeight = grain * detail * .00018;
vec3 floorDx = dFdx(-vViewPosition);
vec3 floorDy = dFdy(-vViewPosition);
vec3 floorR1 = cross(floorDy, normal);
vec3 floorR2 = cross(normal, floorDx);
float floorDet = dot(floorDx, floorR1);
vec3 floorGradient = sign(floorDet)
  * (dFdx(floorHeight) * floorR1 + dFdy(floorHeight) * floorR2);
normal = normalize(max(abs(floorDet), 1e-10) * normal - floorGradient);
