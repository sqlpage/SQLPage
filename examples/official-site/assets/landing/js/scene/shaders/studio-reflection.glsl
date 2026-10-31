// Rough planar reflection of the actual sculpture, in linear light. The
// Gaussian filtering and surface relief soften the blue bands on stone.
vec2 reflectedUV = vFloorReflection.xy / vFloorReflection.w;
reflectedUV += (vec2(grain, coarseGrain) - .5) * .002;
vec4 floorReflection = texture2D(uFloorReflection, reflectedUV);
float reflectionWeight = .48 * floorReflection.a;
outgoingLight = outgoingLight * (1.0 - reflectionWeight)
  + floorReflection.rgb * .48;
outgoingLight *= 1.0 - contactShadow * .85;
#include <opaque_fragment>
