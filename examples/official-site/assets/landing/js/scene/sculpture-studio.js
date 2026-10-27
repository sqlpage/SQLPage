/** A horizontal studio floor, lit by the same softboxes as the sculpture. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { STUDIO_RADIANCE_SCALE } from "./sculpture-assets.js";

export function createSculptureStudio(environment, shaders) {
  const group = new THREE.Group();
  const geometry = new THREE.PlaneGeometry(12, 12);
  geometry.rotateX(-Math.PI / 2);
  const contact = { value: new THREE.Vector2() };
  const material = new THREE.MeshStandardMaterial({
    color: "#263743",
    roughness: 0.56,
    metalness: 0.18,
    envMap: environment,
    envMapIntensity: 0.35 * STUDIO_RADIANCE_SCALE,
    transparent: true,
    depthWrite: false,
  });
  material.onBeforeCompile = (shader) => {
    shader.uniforms.uStudioContact = contact;
    shader.vertexShader = shader.vertexShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vStudioPosition;",
      )
      .replace(
        "#include <begin_vertex>",
        "#include <begin_vertex>\nvStudioPosition = position;",
      );
    shader.fragmentShader = shader.fragmentShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vStudioPosition;\nuniform vec2 uStudioContact;",
      )
      .replace("#include <roughnessmap_fragment>", shaders.studioFinish)
      .replace("#include <normal_fragment_maps>", shaders.studioNormal);
  };
  material.customProgramCacheKey = () => "sqlpage-studio-floor-v2";
  const floor = new THREE.Mesh(geometry, material);
  floor.receiveShadow = true;
  group.add(floor);

  // Local blue spill grazes the floor, revealing its satin texture. It stays
  // in the room when the model turns; there is no illuminated vertical wall.
  const spill = new THREE.PointLight(0x8bb8e8, 3, 6, 2);
  spill.position.set(0.4, 1.2, 0.3);
  group.add(spill);
  const edge = new THREE.PointLight(0xb5d6f4, 1.5, 5, 2);
  edge.position.set(-1.4, 1.4, -0.8);
  group.add(edge);

  const meshes = [];
  const previous = new THREE.Matrix4().multiplyScalar(0);
  const point = new THREE.Vector3();
  const lowest = new THREE.Vector3();
  return {
    group,
    dispose() {
      geometry.dispose();
      material.dispose();
      group.removeFromParent();
    },
    update(model) {
      if (previous.equals(model.matrixWorld)) return;
      previous.copy(model.matrixWorld);
      if (!meshes.length)
        model.traverse((mesh) => {
          if (mesh.isMesh && mesh.castShadow) meshes.push(mesh);
        });
      // A transformed bounding box includes empty corners below a tilted
      // cylinder. Use its actual surface so the sculpture touches the floor,
      // including after a drag, without rotating the floor with the object.
      lowest.set(0, Infinity, 0);
      for (const mesh of meshes) {
        const positions = mesh.geometry.attributes.position;
        for (let i = 0; i < positions.count; i++) {
          point
            .fromBufferAttribute(positions, i)
            .applyMatrix4(mesh.matrixWorld);
          if (point.y < lowest.y) lowest.copy(point);
        }
      }
      group.position.y = lowest.y - 0.005;
      contact.value.set(lowest.x, lowest.z);
    },
  };
}
