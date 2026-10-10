// Soft circular point sprites; no star image or texture fetch is needed.
varying float brightness;
void main() {
  float r = length(gl_PointCoord - .5);
  if (r > .5) discard;
  float a = pow(1.0 - r * 2.0, 2.0) * brightness * .45;
  gl_FragColor = vec4(vec3(.13, .39, .48), a);
}
