// The floor keeps Three's physical lighting and shadow receiver. Fine, filtered
// machining breaks up the softbox reflections without a texture download.
#include <roughnessmap_fragment>
vec2 floorPoint = vStudioPosition.xz;
vec2 floorUV = floorPoint * 260.0;
vec2 footprint = fwidth(floorUV);
float detail = 1.0 - smoothstep(.4, 1.4, max(footprint.x, footprint.y));
float grain = sin(floorUV.x + sin(floorUV.y * 1.37) * 2.0)
  * sin(floorUV.y + sin(floorUV.x * .73) * 2.0);
roughnessFactor = clamp(roughnessFactor + grain * detail * .06, .5, .75);
diffuseColor.rgb *= .97 + grain * detail * .03;
// The contact occlusion is narrow and attached to the lowest surface point;
// directional shadows supply the broader, physically projected silhouette.
vec2 contactOffset = (floorPoint - uStudioContact) / vec2(.48, .38);
float contactShadow = exp(-dot(contactOffset, contactOffset) * 2.0);
diffuseColor.rgb *= 1.0 - contactShadow * .85;
// Fade radially to transparent in every direction. A horizontal floor cannot
// produce the gray pillar that a translucent cyclorama paints on light pages.
vec2 floorOffset = (floorPoint - uStudioContact * .35) / vec2(2.5, 2.0);
float floorRadius = length(floorOffset);
diffuseColor.a *= exp(-floorRadius * floorRadius * 3.8)
  * (1.0 - smoothstep(.75, 1.0, floorRadius));
