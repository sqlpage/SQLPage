// Deterministic point sizes and phases are supplied by sculpture-decoration.
attribute float pointSize;
attribute float phase;
varying float brightness;
uniform float uTime;
void main() {
  vec3 p = position;
  p.y += sin(uTime * .12 + phase) * .06;
  vec4 mv = modelViewMatrix * vec4(p, 1.0);
  gl_Position = projectionMatrix * mv;
  gl_PointSize = clamp(pointSize * (7.0 / -mv.z), 1.0, 12.0);
  brightness = .5 + .35 * sin(phase + uTime * .35);
}
