import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";

/**
 * A local, finite-emitter approximation for this sculpture's illuminated wires.
 * It adds linear radiance through the lid's existing physical BRDF, before ACES.
 * It needs no reflection image, texture, mirror camera, or extra draw call.
 *
 * Based on Karis, Real Shading in Unreal Engine 4, equations 15, 19 and 20:
 * https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf
 * Tube representative points and normalization are approximations; this does
 * not simulate facet refraction, mutual wire occlusion, or multiple reflections.
 */
export function createFiniteJewelLight(edges, shaders) {
  if (!edges.length || edges.length > 48)
    throw new Error("Unexpected jewel edge count");
  const first = edges.map((edge) => new THREE.Vector4(...edge.a, edge.radius));
  const second = edges.map((edge) => new THREE.Vector3(...edge.b));
  const sourceFirst = edges.map((edge) => new THREE.Vector3(...edge.a));
  const sourceSecond = edges.map((edge) => new THREE.Vector3(...edge.b));
  const diamondToView = new THREE.Matrix4();
  const scale = new THREE.Vector3();
  const point = new THREE.Vector3();
  const radiance = new THREE.Color();
  const uniforms = {
    uJewelFirst: { value: first },
    uJewelSecond: { value: second },
    uJewelRadiance: { value: radiance },
  };
  const lastSource = new THREE.Matrix4().multiplyScalar(0);

  return {
    /** Call after finishMetal so both machining and reflection survive. */
    apply(material) {
      const previousCompile = material.onBeforeCompile;
      const previousKey = material.customProgramCacheKey.bind(material);
      const cacheKey = previousKey();
      material.defines = {
        ...material.defines,
        JEWEL_EDGE_COUNT: edges.length,
      };
      material.onBeforeCompile = (shader, renderer) => {
        previousCompile.call(material, shader, renderer);
        Object.assign(shader.uniforms, uniforms);
        shader.fragmentShader = shader.fragmentShader
          .replace(
            "#include <lights_physical_pars_fragment>",
            `#include <lights_physical_pars_fragment>\n${shaders.jewelLight}`,
          )
          .replace("#include <lights_fragment_end>", shaders.jewelReflection);
      };
      material.customProgramCacheKey = () =>
        `${cacheKey}:finite-jewel-${edges.length}-v6`;
      material.needsUpdate = true;
    },

    /** Call after scene.updateMatrixWorld(), before renderer.render(). */
    update(diamond, camera, wireMaterial) {
      radiance
        .copy(wireMaterial.emissive)
        .multiplyScalar(wireMaterial.emissiveIntensity);
      diamondToView.multiplyMatrices(
        camera.matrixWorldInverse,
        diamond.matrixWorld,
      );
      if (diamondToView.equals(lastSource)) return;
      lastSource.copy(diamondToView);
      diamond.getWorldScale(scale);
      // The sculpture uses a uniform scale. Conservative averaging also keeps
      // the source stable if its surrounding placement is adjusted per device.
      const radiusScale =
        (Math.abs(scale.x) + Math.abs(scale.y) + Math.abs(scale.z)) / 3;
      edges.forEach((edge, i) => {
        point.copy(sourceFirst[i]).applyMatrix4(diamondToView);
        first[i].set(point.x, point.y, point.z, edge.radius * radiusScale);
        second[i].copy(sourceSecond[i]).applyMatrix4(diamondToView);
      });
    },
  };
}
