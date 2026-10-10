/** PBR material tuning; shader snippets preserve Three's lighting pipeline. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { STUDIO_RADIANCE_SCALE, STUDIO_URL } from "./sculpture-assets.js";

// The studio's GGX convolution is baked once, offline. CubeUV is already
// filtered for roughness; Three never generates a PMREM during startup.
export async function loadMetalEnvironment() {
  const texture = await new THREE.TextureLoader().loadAsync(STUDIO_URL);
  texture.name = "SQLPage prefiltered dark studio";
  texture.mapping = THREE.CubeUVReflectionMapping;
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.flipY = false;
  texture.minFilter = texture.magFilter = THREE.LinearFilter;
  texture.generateMipmaps = false;
  texture.needsUpdate = true;
  return texture;
}

function localPosition(shader, varying) {
  shader.vertexShader = shader.vertexShader
    .replace("#include <common>", `#include <common>\nvarying vec3 ${varying};`)
    .replace(
      "#include <begin_vertex>",
      `#include <begin_vertex>\n${varying} = position;`,
    );
  shader.fragmentShader = shader.fragmentShader.replace(
    "#include <common>",
    `#include <common>\nvarying vec3 ${varying};`,
  );
}

export function finishMetal(material, environment, shaders) {
  const lid = material.name.includes("Mirror Top");
  const rim = material.name.includes("Gunmetal");
  material.color.set(lid ? "#23303a" : rim ? "#3a4651" : "#252c33");
  material.metalness = 0.96;
  // Keep the cap polished: the satin shell roughness blurs the reflected wires.
  material.roughness = lid ? 0.09 : rim ? 0.23 : 0.34;
  material.envMap = environment;
  material.envMapIntensity =
    (lid ? 0.62 : rim ? 0.76 : 0.68) * STUDIO_RADIANCE_SCALE;
  material.defines = { ...material.defines, METAL_LID: lid ? 1 : 0 };
  material.onBeforeCompile = (shader) => {
    localPosition(shader, "vMetalPosition");
    shader.fragmentShader = shader.fragmentShader.replace(
      "#include <roughnessmap_fragment>",
      shaders.metalFinish,
    );
  };
  material.customProgramCacheKey = () =>
    `sqlpage-black-metal-${lid ? "lid" : "shell"}-v4`;
}

export function finishLiquid(material, clock, environment, interior, shaders) {
  const rim = material.name.includes("Rims");
  material.color.set(rim ? "#133340" : "#06131c");
  material.emissive.set(rim ? "#246582" : "#14638a");
  material.emissiveIntensity = rim ? 0.28 : 0.55;
  material.metalness = 0.2;
  material.roughness = rim ? 0.32 : 0.25;
  material.envMap = environment;
  material.envMapIntensity = 0.18 * STUDIO_RADIANCE_SCALE;
  material.transparent = false;
  material.opacity = 1;
  material.depthWrite = true;
  material.onBeforeCompile = (shader) => {
    localPosition(shader, "vLiquidPosition");
    shader.uniforms.uLiquidTime = clock;
    Object.assign(shader.uniforms, interior);
    shader.vertexShader = shader.vertexShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vInteriorPosition;",
      )
      .replace(
        "#include <begin_vertex>",
        "#include <begin_vertex>\nvInteriorPosition = (modelMatrix * vec4(position, 1.0)).xyz;",
      );
    shader.fragmentShader = shader.fragmentShader.replace(
      "#include <common>",
      `#include <common>\nuniform float uLiquidTime;\n${shaders.liquidFlow}\n${shaders.interiorLight}`,
    );
    shader.fragmentShader = shader.fragmentShader.replace(
      "#include <emissivemap_fragment>",
      shaders.liquidFinish,
    );
  };
  material.customProgramCacheKey = () => "sqlpage-slow-blue-liquid-v5";
}
