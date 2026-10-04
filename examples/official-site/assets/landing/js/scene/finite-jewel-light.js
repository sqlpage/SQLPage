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
export function createFiniteJewelLight(edges) {
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
  const lastSource = new THREE.Matrix4().set(
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
  );

  const glsl = /* glsl */ `
    uniform vec4 uJewelFirst[${edges.length}];
    uniform vec3 uJewelSecond[${edges.length}];
    uniform vec3 uJewelRadiance;

    vec3 jewelTubeLight(
      vec3 lightStart, vec3 lightEnd, float sourceRadius,
      vec3 point, vec3 N, vec3 V, vec3 R,
      PhysicalMaterial surface, float alpha
    ) {
      vec3 L0 = lightStart - point;
      vec3 L1 = lightEnd - point;
      vec3 Ld = L1 - L0;
      float len2 = dot(Ld, Ld);
      float len = sqrt(max(len2, 1e-10));
      float d0 = max(length(L0), 1e-5);
      float d1 = max(length(L1), 1e-5);
      vec3 direction0 = L0 / d0;
      vec3 direction1 = L1 / d1;
      float NoL0 = dot(N, direction0);
      float NoL1 = dot(N, direction1);
      if (max(NoL0, NoL1) <= 0.0) return vec3(0.0);

      // Closest point of the finite source centreline to the reflection ray.
      // Unlike an image on a disk, this follows the camera and the 3D source.
      float rd = dot(R, Ld);
      float r0 = dot(R, L0);
      float denominator = max(len2 - rd * rd, len2 * 1e-6);
      float t = clamp((r0 * rd - dot(L0, Ld)) / denominator, 0.0, 1.0);
      vec3 representative = L0 + t * Ld;
      if (dot(R, representative) < 0.0) {
        // Equations treating R as an infinite line have a discontinuity when
        // the emitter lies behind the ray. Explicit endpoint selection fixes it.
        representative = dot(R, L0) / d0 > dot(R, L1) / d1 ? L0 : L1;
      }

      // Move from the centreline toward the finite cylindrical surface.
      vec3 toRay = max(dot(representative, R), 0.0) * R - representative;
      float rayDistance = max(length(toRay), 1e-6);
      representative += toRay * min(sourceRadius / rayDistance, 1.0);
      float distance = max(length(representative), sourceRadius + 1e-5);
      vec3 L = representative / distance;
      if (dot(N, L) <= 0.0) return vec3(0.0);

      vec3 mid = .5 * (L0 + L1);
      float midDistance = max(length(mid), sourceRadius + 1e-5);
      vec3 midDirection = mid / midDistance;
      vec3 axis = Ld / len;
      float projectedExtent = sqrt(max(1.0 - pow(dot(axis, midDirection), 2.0), 0.0));

      // Integral of N.L / |L|^3 along the finite segment, times its projected
      // tube width. The length factor preserves radiance as geometry scales.
      float lineIrradiance = max(NoL0 + NoL1, 0.0) * len /
        max(d0 * d1 + dot(L0, L1), 1e-6);
      float irradiance = lineIrradiance * (2.0 * sourceRadius * projectedExtent);
      irradiance += PI * sourceRadius * sourceRadius /
        (midDistance * midDistance) * max(dot(N, midDirection), 0.0);

      // Normalize in half-vector space, where the microfacet GGX distribution
      // is defined. This includes stretching at grazing angles instead of
      // retaining the energy normalization of a head-on, isotropic mirror.
      float halfSpan = length(normalize(direction0 + V) - normalize(direction1 + V));
      float alphaLine = min(1.0, alpha + halfSpan * (2.0 / 3.0));
      float alphaSphere = min(1.0, alpha + sourceRadius / (3.0 * distance));
      float alphaSphereAcross = min(1.0, alpha + sourceRadius /
        (3.0 * distance * max(dot(N, V), .08)));
      float normalization = (alpha / max(alphaLine, 1e-5)) *
        alpha * alpha / max(alphaSphere * alphaSphereAcross, 1e-10);
      return BRDF_GGX(L, V, N, surface) * irradiance * normalization;
    }
  `;

  return {
    /** Call after finishMetal so both machining and reflection survive. */
    apply(material) {
      const previousCompile = material.onBeforeCompile;
      const previousKey = material.customProgramCacheKey.bind(material);
      const cacheKey = previousKey();
      material.onBeforeCompile = (shader, renderer) => {
        previousCompile.call(material, shader, renderer);
        Object.assign(shader.uniforms, uniforms);
        shader.fragmentShader = shader.fragmentShader
          .replace(
            "#include <lights_physical_pars_fragment>",
            `#include <lights_physical_pars_fragment>\n${glsl}`,
          )
          .replace(
            "#include <lights_fragment_end>",
            /* glsl */ `
          #include <lights_fragment_end>
          vec3 jewelRay = reflect(-geometryViewDir, geometryNormal);
          // Variance from one screen pixel prevents subpixel specular shimmer.
          // No resolution-sized texture is involved. Normal material roughness
          // dominates at normal viewing sizes; only distant pixels are widened.
          float jewelPixel = max(length(dFdx(jewelRay)), length(dFdy(jewelRay)));
          float jewelAlpha = sqrt(pow(material.roughness, 4.0) + .25 * jewelPixel * jewelPixel);
          jewelAlpha = clamp(jewelAlpha, .012, 1.0);
          PhysicalMaterial jewelSurface = material;
          jewelSurface.roughness = sqrt(jewelAlpha);
          vec3 jewelSpecular = vec3(0.0);
          for (int edge = 0; edge < ${edges.length}; edge++) {
            jewelSpecular += jewelTubeLight(
              uJewelFirst[edge].xyz, uJewelSecond[edge], uJewelFirst[edge].w,
              geometryPosition, geometryNormal, geometryViewDir, jewelRay,
              jewelSurface, jewelAlpha
            );
          }
          // Overlapping tubes at a junction cannot reflect more radiance than
          // a whole hemisphere filled by that emitter. Reuse the same material
          // BRDF integral to bound the finite-source approximation's hot spots.
          vec3 jewelEnergyLimit = EnvironmentBRDF(
            geometryNormal, geometryViewDir, material.specularColorBlended,
            material.specularF90, material.roughness
          );
          reflectedLight.directSpecular += uJewelRadiance * min(jewelSpecular, jewelEnergyLimit);
        `,
          );
      };
      material.customProgramCacheKey = () =>
        `${cacheKey}:finite-jewel-${edges.length}-v5`;
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
