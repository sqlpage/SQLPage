// Two coherent noise scales move through the separator bands as slow currents.
float liquidHash(vec3 p) {
  p = fract(p * .3183099 + vec3(.1, .2, .3));
  p *= 17.0;
  return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}
float liquidNoise(vec3 p) {
  vec3 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(mix(liquidHash(i), liquidHash(i + vec3(1,0,0)), f.x),
      mix(liquidHash(i + vec3(0,1,0)), liquidHash(i + vec3(1,1,0)), f.x), f.y),
    mix(mix(liquidHash(i + vec3(0,0,1)), liquidHash(i + vec3(1,0,1)), f.x),
      mix(liquidHash(i + vec3(0,1,1)), liquidHash(i + vec3(1,1,1)), f.x), f.y),
    f.z);
}
float liquidFlow(vec3 position, float time) {
  vec3 p = position * vec3(3.2, 14.0, 3.2);
  vec3 current = vec3(time * .075, sin(time * .09) * .17, -time * .055);
  float drift = liquidNoise(p + current);
  float eddy = liquidNoise(p * 1.9 - current * .6 + drift * 1.5);
  return smoothstep(.28, .78, drift * .65 + eddy * .35);
}
