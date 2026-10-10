// Separable Gaussian blur; extraction removes dim surfaces from the first pass.
uniform sampler2D tSource;
uniform vec2 uStep;
uniform float uExtract;
varying vec2 vUv;
vec3 sampleLight(vec2 uv) {
  vec3 c = texture2D(tSource, uv).rgb;
  float luminance = dot(c, vec3(.2126, .7152, .0722));
  return c * mix(1.0, smoothstep(.045, .065, luminance), uExtract);
}
void main() {
  vec3 light = sampleLight(vUv) * .2270270270;
  light += (sampleLight(vUv + uStep * 1.3846153846)
    + sampleLight(vUv - uStep * 1.3846153846)) * .3162162162;
  light += (sampleLight(vUv + uStep * 3.2307692308)
    + sampleLight(vUv - uStep * 3.2307692308)) * .0702702703;
  gl_FragColor = vec4(light, 1.0);
}
