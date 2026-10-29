/** A horizontal studio floor, lit by the same softboxes as the sculpture. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { Reflector } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/objects/Reflector.js/+esm";
import { FullScreenQuad } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/postprocessing/Pass.js/+esm";
import { STUDIO_RADIANCE_SCALE } from "./sculpture-assets.js";

export function createSculptureStudio(environment, shaders) {
  const group = new THREE.Group();
  const geometry = new THREE.PlaneGeometry(20, 20);
  // The reflected camera renders at 512 × 512; the floor filters
  // that linear HDR image before the main scene's single tone-mapping pass.
  const reflection = new Reflector(geometry, {
    textureWidth: 512,
    textureHeight: 512,
    multisample: 0,
    clipBias: 0.002,
  });
  reflection.rotation.x = -Math.PI / 2;
  reflection.visible = false;
  group.add(reflection);
  const blurred = Array.from(
    { length: 2 },
    () =>
      new THREE.WebGLRenderTarget(256, 256, {
        type: THREE.HalfFloatType,
        depthBuffer: false,
      }),
  );
  const blur = new THREE.ShaderMaterial({
    uniforms: {
      tSource: { value: reflection.getRenderTarget().texture },
      uStep: { value: new THREE.Vector2() },
      uExtract: { value: 0 },
    },
    vertexShader: shaders.fullscreenVertex,
    fragmentShader: shaders.blurFragment,
    depthTest: false,
    depthWrite: false,
    toneMapped: false,
  });
  const blurQuad = new FullScreenQuad(blur);
  const contact = { value: new THREE.Vector2() };
  const material = new THREE.MeshStandardMaterial({
    color: "#1c2932",
    roughness: 0.48,
    metalness: 0.3,
    envMap: environment,
    envMapIntensity: 0.35 * STUDIO_RADIANCE_SCALE,
    transparent: true,
    depthWrite: false,
  });
  material.onBeforeCompile = (shader) => {
    shader.uniforms.uStudioContact = contact;
    shader.uniforms.uFloorReflection = { value: blurred[1].texture };
    shader.uniforms.uFloorReflectionMatrix =
      reflection.material.uniforms.textureMatrix;
    shader.vertexShader = shader.vertexShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vStudioPosition;\nvarying vec4 vFloorReflection;\nuniform mat4 uFloorReflectionMatrix;",
      )
      .replace(
        "#include <begin_vertex>",
        "#include <begin_vertex>\nvStudioPosition = vec3(position.x, 0.0, -position.y);\nvFloorReflection = uFloorReflectionMatrix * vec4(position, 1.0);",
      );
    shader.fragmentShader = shader.fragmentShader
      .replace(
        "#include <common>",
        `#include <common>\nvarying vec3 vStudioPosition;\nuniform vec2 uStudioContact;\nvarying vec4 vFloorReflection;\nuniform sampler2D uFloorReflection;\n${shaders.studioNoise}`,
      )
      .replace("#include <roughnessmap_fragment>", shaders.studioFinish)
      .replace("#include <normal_fragment_maps>", shaders.studioNormal)
      .replace("#include <opaque_fragment>", shaders.studioReflection);
  };
  material.customProgramCacheKey = () => "sqlpage-studio-floor-v3";
  const floor = new THREE.Mesh(geometry, material);
  floor.rotation.x = -Math.PI / 2;
  floor.receiveShadow = true;
  group.add(floor);

  // Local blue spill grazes the floor, revealing its satin texture. It stays
  // in the room when the model turns; there is no illuminated vertical wall.
  const spill = new THREE.RectAreaLight(0xc3ddf8, 5, 3, 1.5);
  spill.position.set(-1.5, 2, -1.3);
  spill.lookAt(0, 0, 0.2);
  group.add(spill);
  const edge = new THREE.RectAreaLight(0x8abaf0, 2.5, 2, 2);
  edge.position.set(1.8, 2, -1.5);
  edge.lookAt(0, 0, 0.2);
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
      reflection.dispose();
      blurred.forEach((target) => {
        target.dispose();
      });
      blur.dispose();
      blurQuad.dispose();
      group.removeFromParent();
    },
    renderReflection(renderer, scene, camera) {
      const savedTarget = renderer.getRenderTarget();
      floor.visible = false;
      try {
        reflection.onBeforeRender(renderer, scene, camera);
        // A continuous Gaussian footprint avoids repeated, offset copies of
        // the luminous bands. RGB and coverage are filtered together.
        blur.uniforms.tSource.value = reflection.getRenderTarget().texture;
        blur.uniforms.uStep.value.set(1.2 / 512, 0);
        renderer.setRenderTarget(blurred[0]);
        blurQuad.render(renderer);
        blur.uniforms.tSource.value = blurred[0].texture;
        blur.uniforms.uStep.value.set(0, 0.8 / 256);
        renderer.setRenderTarget(blurred[1]);
        blurQuad.render(renderer);
      } finally {
        renderer.setRenderTarget(savedTarget);
        floor.visible = true;
        reflection.visible = false;
      }
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
