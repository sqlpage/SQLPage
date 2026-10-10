// Inject after direct lighting; JEWEL_EDGE_COUNT specializes the uniform arrays.
#include <lights_fragment_end>
vec3 jewelRay = reflect(-geometryViewDir, geometryNormal);
// Variance from one screen pixel prevents subpixel specular shimmer.
// No resolution-sized texture is involved. Normal material roughness
// dominates at normal viewing sizes; only distant pixels are widened.
float jewelPixel = max(length(dFdx(jewelRay)), length(dFdy(jewelRay)));
float jewelAlpha = sqrt(pow(material.roughness, 4.0) + .25 * jewelPixel * jewelPixel);
// Alpha is roughness squared. A .012 floor would turn the
// polished .09 cap back into a .11-roughness surface. Retain
// its mirror image while the derivative term filters distant pixels.
jewelAlpha = clamp(jewelAlpha, .0036, 1.0);
PhysicalMaterial jewelSurface = material;
jewelSurface.roughness = sqrt(jewelAlpha);
vec3 jewelSpecular = vec3(0.0);
for (int edge = 0; edge < JEWEL_EDGE_COUNT; edge++) {
  jewelSpecular += jewelTubeLight(
    uJewelFirst[edge].xyz, uJewelSecond[edge], uJewelFirst[edge].w,
    geometryPosition, geometryNormal, geometryViewDir, jewelRay,
    jewelSurface, jewelAlpha
  );
}
// Overlapping tubes at a junction cannot reflect more radiance than
// a whole hemisphere filled by that emitter. Reuse the same material
// BRDF integral to bound the finite-source approximation's hot spots.
vec3 jewelEnergyLimit = EnvironmentBRDF(
  geometryNormal, geometryViewDir, material.specularColorBlended,
  material.specularF90, material.roughness
);
reflectedLight.directSpecular += uJewelRadiance * min(jewelSpecular, jewelEnergyLimit);
