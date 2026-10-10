// Keep model coordinates for flow and world coordinates for the interior emitter.
varying vec3 vPosition;
varying vec3 vInteriorPosition;
void main() {
  vPosition = position;
  vInteriorPosition = (modelMatrix * vec4(position, 1.0)).xyz;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
