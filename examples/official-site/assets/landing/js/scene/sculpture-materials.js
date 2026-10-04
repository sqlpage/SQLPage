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

export function finishMetal(material, environment) {
  const lid = material.name.includes("Mirror Top");
  const rim = material.name.includes("Gunmetal");
  material.color.set(lid ? "#23303a" : rim ? "#3a4651" : "#252c33");
  material.metalness = 0.96;
  material.roughness = lid ? 0.19 : rim ? 0.23 : 0.34;
  material.envMap = environment;
  material.envMapIntensity =
    (lid ? 0.62 : rim ? 0.76 : 0.68) * STUDIO_RADIANCE_SCALE;
  material.onBeforeCompile = (shader) => {
    localPosition(shader, "vMetalPosition");
    shader.fragmentShader = shader.fragmentShader.replace(
      "#include <roughnessmap_fragment>",
      `
      #include <roughnessmap_fragment>
      float metalGrain = fract(sin(dot(floor(vMetalPosition * vec3(190.0, 1250.0, 190.0)), vec3(12.9898, 78.233, 43.128))) * 43758.5453);
      float toolPath = ${lid ? "length(vMetalPosition.xz)" : "vMetalPosition.y"} * 760.0;
      float machining = sin(toolPath) * (1.0 - smoothstep(.6, 3.0, fwidth(toolPath)));
      roughnessFactor = clamp(roughnessFactor + machining * .012 + (metalGrain - .5) * .016, .15, .48);
    `,
    );
  };
  material.customProgramCacheKey = () =>
    `sqlpage-black-metal-${lid ? "lid" : "shell"}-v3`;
}

export const LIQUID_FLOW_GLSL = `
  float liquidHash(vec3 p) { p = fract(p * .3183099 + vec3(.1, .2, .3)); p *= 17.0; return fract(p.x * p.y * p.z * (p.x + p.y + p.z)); }
  float liquidNoise(vec3 p) {
    vec3 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(liquidHash(i), liquidHash(i + vec3(1,0,0)), f.x), mix(liquidHash(i + vec3(0,1,0)), liquidHash(i + vec3(1,1,0)), f.x), f.y),
      mix(mix(liquidHash(i + vec3(0,0,1)), liquidHash(i + vec3(1,0,1)), f.x), mix(liquidHash(i + vec3(0,1,1)), liquidHash(i + vec3(1,1,1)), f.x), f.y), f.z);
  }
  float liquidFlow(vec3 position, float time) {
    vec3 p = position * vec3(3.2, 14.0, 3.2);
    vec3 current = vec3(time * .075, sin(time * .09) * .17, -time * .055);
    float drift = liquidNoise(p + current);
    float eddy = liquidNoise(p * 1.9 - current * .6 + drift * 1.5);
    return smoothstep(.28, .78, drift * .65 + eddy * .35);
  }
`;

// The opaque shell hides the emitter; the separator bands carry its soft glow.
export const INTERIOR_LIGHT_GLSL = `
  uniform vec3 uInteriorPosition;
  uniform float uInteriorRadius;
  uniform float uInteriorPulse;
  varying vec3 vInteriorPosition;
  float interiorLight() {
    vec3 offset = (vInteriorPosition - uInteriorPosition) / uInteriorRadius;
    return 1.0 + uInteriorPulse * exp(-dot(offset, offset) * 1.8);
  }
`;

export function finishLiquid(material, clock, environment, interior) {
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
      `#include <common>\nuniform float uLiquidTime;\n${LIQUID_FLOW_GLSL}\n${INTERIOR_LIGHT_GLSL}`,
    );
    shader.fragmentShader = shader.fragmentShader.replace(
      "#include <emissivemap_fragment>",
      `
      #include <emissivemap_fragment>
      float flow = liquidFlow(vLiquidPosition, uLiquidTime);
      diffuseColor.rgb *= .65 + flow * .35;
      totalEmissiveRadiance *= (.42 + flow * .58) * interiorLight();
    `,
    );
  };
  material.customProgramCacheKey = () => "sqlpage-slow-blue-liquid-v5";
}
