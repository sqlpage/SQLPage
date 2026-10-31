// Place the cropped glow buffer back into its viewport rectangle.
varying vec2 vUv;
uniform vec4 uRect;
void main() {
  vUv = uv;
  gl_Position = vec4((uRect.xy + uv * uRect.zw) * 2.0 - 1.0, 0.0, 1.0);
}
