// Shared liquid-flow and interior-light snippets are prepended by selective-glow.
uniform vec3 uEmission;
uniform float uLiquid;
uniform float uLiquidTime;
varying vec3 vPosition;
void main() {
  float flow = uLiquid > .5
    ? (.42 + liquidFlow(vPosition, uLiquidTime) * .58) * interiorLight()
    : 1.0;
  gl_FragColor = vec4(uEmission * flow, 1.0);
}
