// Stable value noise gives the studio surface an irregular, non-periodic grain.
float studioHash(vec2 p) {
  vec3 q = fract(vec3(p.xyx) * .1031);
  q += dot(q, q.yzx + 33.33);
  return fract((q.x + q.y) * q.z);
}
float studioNoise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(mix(studioHash(i), studioHash(i + vec2(1, 0)), f.x),
    mix(studioHash(i + vec2(0, 1)), studioHash(i + vec2(1, 1)), f.x), f.y);
}
