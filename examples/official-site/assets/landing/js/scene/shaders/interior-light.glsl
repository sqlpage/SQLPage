// The opaque shell hides the emitter; its separator bands carry this soft glow.
uniform vec3 uInteriorPosition;
uniform float uInteriorRadius;
uniform float uInteriorPulse;
varying vec3 vInteriorPosition;
float interiorLight() {
  vec3 offset = (vInteriorPosition - uInteriorPosition) / uInteriorRadius;
  return 1.0 + uInteriorPulse * exp(-dot(offset, offset) * 1.8);
}
