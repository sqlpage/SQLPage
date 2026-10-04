import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { FullScreenQuad } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/postprocessing/Pass.js/+esm";
import {
  INTERIOR_LIGHT_GLSL,
  LIQUID_FLOW_GLSL,
} from "./sculpture-materials.js";

const vertex = `varying vec2 vUv;
  void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;

// Only luminous geometry enters this cropped buffer. Two blur scales retain
// a crisp core and a quiet halo, without full-viewport bloom or runtime mips.
export function createSelectiveGlow(renderer, scene, camera, clock, interior) {
  const source = new THREE.WebGLRenderTarget(1, 1, {
    type: THREE.HalfFloatType,
    depthBuffer: true,
    samples: Math.min(2, renderer.capabilities.maxSamples),
  });
  source.texture.name = "SQLPage isolated emission";
  const buffers = Array.from(
    { length: 4 },
    () =>
      new THREE.WebGLRenderTarget(1, 1, {
        type: THREE.HalfFloatType,
        depthBuffer: false,
      }),
  );
  for (const target of [source, ...buffers])
    target.texture.generateMipmaps = false;
  const [narrowH, narrowV, wideH, wideV] = buffers;
  const maskCamera = camera.clone();
  const black = new THREE.MeshBasicMaterial({ color: 0x000000 });
  const omitted = new THREE.MeshBasicMaterial({ visible: false });
  const glowMaterials = new Map();
  const glowMaterial = (original) => {
    if (
      !(original instanceof THREE.MeshStandardMaterial) ||
      !/Jewel Wire|Endpoint|Separator/.test(original.name)
    )
      return original.transparent ? omitted : black;
    const cached = glowMaterials.get(original);
    if (cached) return cached;
    const liquid = original.name.includes("Separator");
    const material = new THREE.ShaderMaterial({
      name: `${original.name} — emission only`,
      uniforms: {
        ...interior,
        uEmission: {
          value: original.emissive
            .clone()
            .multiplyScalar(original.emissiveIntensity * (liquid ? 3 : 1)),
        },
        uLiquid: { value: liquid ? 1 : 0 },
        uLiquidTime: clock,
      },
      vertexShader: `varying vec3 vPosition; varying vec3 vInteriorPosition;
        void main() { vPosition = position; vInteriorPosition = (modelMatrix * vec4(position, 1.0)).xyz; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }`,
      fragmentShader: `uniform vec3 uEmission; uniform float uLiquid; uniform float uLiquidTime; varying vec3 vPosition;
        ${LIQUID_FLOW_GLSL}
        ${INTERIOR_LIGHT_GLSL}
        void main() {
          float flow = uLiquid > .5 ? (.42 + liquidFlow(vPosition, uLiquidTime) * .58) * interiorLight() : 1.0;
          gl_FragColor = vec4(uEmission * flow, 1.0);
        }`,
      toneMapped: false,
      side: original.side,
    });
    glowMaterials.set(original, material);
    return material;
  };

  const blur = new THREE.ShaderMaterial({
    uniforms: {
      tSource: { value: source.texture },
      uStep: { value: new THREE.Vector2() },
      uExtract: { value: 0 },
    },
    vertexShader: vertex,
    fragmentShader: `uniform sampler2D tSource; uniform vec2 uStep; uniform float uExtract; varying vec2 vUv;
      vec3 sampleLight(vec2 uv) {
        vec3 c = texture2D(tSource, uv).rgb;
        float luminance = dot(c, vec3(.2126, .7152, .0722));
        return c * mix(1.0, smoothstep(.045, .065, luminance), uExtract);
      }
      void main() {
        vec3 light = sampleLight(vUv) * .2270270270;
        light += (sampleLight(vUv + uStep * 1.3846153846) + sampleLight(vUv - uStep * 1.3846153846)) * .3162162162;
        light += (sampleLight(vUv + uStep * 3.2307692308) + sampleLight(vUv - uStep * 3.2307692308)) * .0702702703;
        gl_FragColor = vec4(light, 1.0);
      }`,
    depthTest: false,
    depthWrite: false,
    toneMapped: false,
  });
  const composite = new THREE.ShaderMaterial({
    uniforms: {
      tNarrow: { value: narrowV.texture },
      tWide: { value: wideV.texture },
      uRect: { value: new THREE.Vector4(0, 0, 1, 1) },
    },
    vertexShader: `varying vec2 vUv; uniform vec4 uRect;
      void main() { vUv = uv; gl_Position = vec4((uRect.xy + uv * uRect.zw) * 2.0 - 1.0, 0.0, 1.0); }`,
    fragmentShader: `uniform sampler2D tNarrow; uniform sampler2D tWide; varying vec2 vUv;
      void main() {
        if (any(lessThan(vUv, vec2(0.0))) || any(greaterThan(vUv, vec2(1.0)))) discard;
        vec3 halo = texture2D(tNarrow, vUv).rgb * .64 + texture2D(tWide, vUv).rgb * .16;
        gl_FragColor = vec4(max(halo, vec3(0.0)), 1.0);
        #include <tonemapping_fragment>
        #include <colorspace_fragment>
        vec3 light = clamp(gl_FragColor.rgb * .72, 0.0, 1.0);
        float coverage = max(light.r, max(light.g, light.b));
        gl_FragColor = vec4(light, coverage);
      }`,
    transparent: true,
    premultipliedAlpha: true,
    blending: THREE.NormalBlending,
    depthTest: false,
    depthWrite: false,
  });
  const quad = new FullScreenQuad(blur);
  const clearColor = new THREE.Color();
  let width = 1,
    height = 1,
    ratio = 1,
    cropWidth = 0,
    cropHeight = 0,
    disposed = false;
  const fitCrop = (needed, available, previous) => {
    needed = Math.min(needed, available);
    // Grow with spare room, but shrink only after two buckets. A subpixel
    // oscillation at a boundary must never reallocate five textures each frame.
    return previous === 0 ||
      previous > available ||
      needed > previous ||
      needed < previous - 256
      ? Math.min(available, Math.ceil((needed + 64) / 128) * 128)
      : previous;
  };
  const blurInto = (from, to, x, y, extract = 0) => {
    blur.uniforms.tSource.value = from.texture;
    blur.uniforms.uStep.value.set(x / from.width, y / from.height);
    blur.uniforms.uExtract.value = extract;
    renderer.setRenderTarget(to);
    quad.material = blur;
    quad.render(renderer);
  };
  return {
    setSize(w, h, pixelRatio) {
      width = w;
      height = h;
      ratio = Math.min(pixelRatio, 1.25);
    },
    render(model, exclude, bounds) {
      if (disposed || !model) return;
      // The mesh keeps its full resolution. Only its soft halo is capped;
      // enlarge the guard band to accommodate blur at any effective ratio.
      const minimumRatio = Math.min(ratio, 1152 / Math.max(width, height));
      const padding = Math.max(40, Math.ceil(24 / minimumRatio));
      cropWidth = fitCrop(
        bounds.right - bounds.left + 2 * padding,
        width,
        cropWidth,
      );
      cropHeight = fitCrop(
        bounds.bottom - bounds.top + 2 * padding,
        height,
        cropHeight,
      );
      const left = THREE.MathUtils.clamp(
        Math.round((bounds.left + bounds.right - cropWidth) * 0.5),
        0,
        width - cropWidth,
      );
      const top = THREE.MathUtils.clamp(
        Math.round((bounds.top + bounds.bottom - cropHeight) * 0.5),
        0,
        height - cropHeight,
      );
      const effectiveRatio = Math.min(
        ratio,
        1152 / Math.max(cropWidth, cropHeight),
      );
      const w = Math.max(32, Math.round(cropWidth * effectiveRatio)),
        h = Math.max(32, Math.round(cropHeight * effectiveRatio));
      source.setSize(w, h);
      narrowH.setSize(Math.ceil(w / 2), Math.ceil(h / 2));
      narrowV.setSize(Math.ceil(w / 2), Math.ceil(h / 2));
      wideH.setSize(Math.ceil(w / 4), Math.ceil(h / 4));
      wideV.setSize(Math.ceil(w / 4), Math.ceil(h / 4));
      maskCamera.copy(camera, false);
      maskCamera.setViewOffset(width, height, left, top, cropWidth, cropHeight);
      // The canvas expands from the square landing raster into the viewport.
      // Preserve its displayed aspect while cropping the emission pass.
      maskCamera.aspect = camera.aspect;
      maskCamera.updateProjectionMatrix();
      composite.uniforms.uRect.value.set(
        left / width,
        (height - top - cropHeight) / height,
        cropWidth / width,
        cropHeight / height,
      );
      const savedTarget = renderer.getRenderTarget(),
        savedAutoClear = renderer.autoClear,
        savedAlpha = renderer.getClearAlpha();
      renderer.getClearColor(clearColor);
      const hidden = exclude
        .filter((object) => !!object)
        .map((object) => ({ object, visible: object.visible }));
      const meshes = [];
      try {
        hidden.forEach(({ object }) => {
          object.visible = false;
        });
        model.traverse((object) => {
          if (!(object instanceof THREE.Mesh) || !object.visible) return;
          meshes.push({ mesh: object, material: object.material });
          object.material = Array.isArray(object.material)
            ? object.material.map(glowMaterial)
            : glowMaterial(object.material);
        });
        renderer.autoClear = true;
        renderer.setClearColor(0x000000, 0);
        renderer.setRenderTarget(source);
        renderer.render(scene, maskCamera);
        blurInto(source, narrowH, 2, 0, 1);
        blurInto(narrowH, narrowV, 0, 1);
        blurInto(narrowV, wideH, 2, 0);
        blurInto(wideH, wideV, 0, 1);
      } finally {
        meshes.forEach(({ mesh, material }) => {
          mesh.material = material;
        });
        hidden.forEach(({ object, visible }) => {
          object.visible = visible;
        });
        renderer.setRenderTarget(savedTarget);
        renderer.setClearColor(clearColor, savedAlpha);
        renderer.autoClear = savedAutoClear;
      }
      renderer.autoClear = false;
      quad.material = composite;
      try {
        quad.render(renderer);
      } finally {
        renderer.autoClear = savedAutoClear;
      }
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      glowMaterials.forEach((material) => {
        material.dispose();
      });
      glowMaterials.clear();
      black.dispose();
      omitted.dispose();
      source.dispose();
      buffers.forEach((target) => {
        target.dispose();
      });
      quad.dispose();
      blur.dispose();
      composite.dispose();
    },
  };
}
