// Tone-map emission once, then derive premultiplied-alpha coverage for the page.
uniform sampler2D tNarrow;
uniform sampler2D tWide;
varying vec2 vUv;
void main() {
  if (any(lessThan(vUv, vec2(0.0))) || any(greaterThan(vUv, vec2(1.0)))) discard;
  vec3 halo = texture2D(tNarrow, vUv).rgb * .64
    + texture2D(tWide, vUv).rgb * .16;
  gl_FragColor = vec4(max(halo, vec3(0.0)), 1.0);
  #include <tonemapping_fragment>
  #include <colorspace_fragment>
  vec3 light = clamp(gl_FragColor.rgb * .72, 0.0, 1.0);
  float coverage = max(light.r, max(light.g, light.b));
  gl_FragColor = vec4(light, coverage);
}
