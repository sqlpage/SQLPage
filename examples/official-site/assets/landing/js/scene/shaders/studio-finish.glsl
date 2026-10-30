// PBR stone: irregular roughness and relief catch the grazing softboxes.
#include <roughnessmap_fragment>
vec2 floorPoint = vStudioPosition.xz;
vec2 floorUV = floorPoint * 220.0;
vec2 footprint = fwidth(floorUV);
float detail = 1.0 - smoothstep(.5, 2.0, max(footprint.x, footprint.y));
float grain = studioNoise(floorUV);
float coarseGrain = studioNoise(floorPoint * 75.0);
roughnessFactor = clamp(roughnessFactor + (coarseGrain - .5) * .16, .32, .6);
diffuseColor.rgb *= .9 + coarseGrain * .15 + grain * detail * .05;
vec2 contactOffset = (floorPoint - uStudioContact) / vec2(.48, .38);
float contactShadow = exp(-dot(contactOffset, contactOffset) * 2.0);
diffuseColor.rgb *= 1.0 - contactShadow * .85;
// A broad horizontal surface remains visible around and behind the sculpture.
// Its distant edges blend into the page without fading away the contact area.
vec2 floorOffset = (floorPoint - uStudioContact * .35) / vec2(4.2, 5.5);
float floorRadius = length(floorOffset);
diffuseColor.a *= 1.0 - smoothstep(.38, 1.0, floorRadius);
// This square renderer moves between page anchors. Fade before its boundary,
// so a broad ground plane never exposes a rectangular crop on the page.
vec2 floorScreen = gl_FragCoord.xy / 1000.0;
vec2 screenGround = (floorScreen - vec2(.49, .17)) / vec2(.5, .3);
diffuseColor.a *= (1.0 - smoothstep(.25, 1.0, length(screenGround)))
  * smoothstep(0.0, .12, floorScreen.y);
