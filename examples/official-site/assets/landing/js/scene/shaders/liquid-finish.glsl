// Inject after Three's emissive-map chunk; preserve the base physical material.
#include <emissivemap_fragment>
float flow = liquidFlow(vLiquidPosition, uLiquidTime);
diffuseColor.rgb *= .65 + flow * .35;
totalEmissiveRadiance *= (.42 + flow * .58) * interiorLight();
