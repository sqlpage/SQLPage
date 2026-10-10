/** Cropped emission mask → two blur scales → alpha composite over the PBR scene. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { FullScreenQuad } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/postprocessing/Pass.js/+esm";

// Only luminous geometry enters this cropped buffer. Two blur scales retain
// a crisp core and a quiet halo, without full-viewport bloom or runtime mips.
export function createSelectiveGlow(
  renderer,
  scene,
  camera,
  clock,
  interior,
  shaders,
) {
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
      vertexShader: shaders.emissionVertex,
      fragmentShader: [
        shaders.liquidFlow,
        shaders.interiorLight,
        shaders.emissionFragment,
      ].join("\n"),
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
    vertexShader: shaders.fullscreenVertex,
    fragmentShader: shaders.blurFragment,
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
    vertexShader: shaders.compositeVertex,
    fragmentShader: shaders.compositeFragment,
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
      // Preserve the reference camera aspect: cropping the emission buffer
      // must not stretch the sculpture projection to the crop dimensions.
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
