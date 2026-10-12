// Inject after Three's roughness-map chunk. METAL_LID selects circular machining.
#include <roughnessmap_fragment>
#if METAL_LID
  float metalToolCoordinate = length(vMetalPosition.xz);
  const float metalMachining = .003;
  const float metalGrainStrength = .004;
  const float metalMinimumRoughness = .06;
#else
  float metalToolCoordinate = vMetalPosition.y;
  const float metalMachining = .012;
  const float metalGrainStrength = .016;
  const float metalMinimumRoughness = .15;
#endif
float metalGrain = fract(sin(dot(
  floor(vMetalPosition * vec3(190.0, 1250.0, 190.0)),
  vec3(12.9898, 78.233, 43.128))) * 43758.5453);
float toolPath = metalToolCoordinate * 760.0;
float machining = sin(toolPath) * (1.0 - smoothstep(.6, 3.0, fwidth(toolPath)));
roughnessFactor = clamp(roughnessFactor + machining * metalMachining
  + (metalGrain - .5) * metalGrainStrength, metalMinimumRoughness, .48);
