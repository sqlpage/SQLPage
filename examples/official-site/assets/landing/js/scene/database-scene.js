/**
 * Owns one square renderer and its resource lifetime. Scroll supplies placement;
 * controls supply local rotation. Materials, studio surfaces and glow stay independent.
 */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { RectAreaLightUniformsLib } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/lights/RectAreaLightUniformsLib.js/+esm";
import { GLTFLoader } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/loaders/GLTFLoader.js/+esm";
import { createFiniteJewelLight } from "./finite-jewel-light.js";
import { JEWEL_EDGES } from "./jewel-edges.js";
import { MODEL_URL } from "./sculpture-assets.js";
import { createSculptureControls } from "./sculpture-controls.js";
import {
  CAMERA_DISTANCE,
  cameraForSculpture,
  DEFAULT_TILT,
} from "./sculpture-layout.js";
import {
  finishLiquid,
  finishMetal,
  loadMetalEnvironment,
} from "./sculpture-materials.js";
import { createSculptureStudio } from "./sculpture-studio.js";
import { createSelectiveGlow } from "./selective-glow.js";
import { loadSculptureShaders } from "./shader-sources.js";

const clamp = THREE.MathUtils.clamp;
/**
 * @typedef {{ motion: boolean, opacity: number, turn?: number, frame: {left: number, top: number, size: number} }} SceneState
 * @typedef {{ zoom(amount: number): void, reset(): void, dispose(): void }} SceneController
 * @param {{ mount: HTMLElement, hitArea: HTMLElement,
 *   getState: () => SceneState, onReady: () => void, onError: () => void }} options
 * @returns {SceneController}
 */
export function createDatabaseScene({
  mount,
  hitArea,
  getState,
  onReady,
  onError,
}) {
  performance.mark("sqlpage:scene-start");
  let renderer;
  try {
    renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
      powerPreference: "high-performance",
    });
  } catch {
    onError();
    return { zoom() {}, reset() {}, dispose() {} };
  }
  let disposed = false,
    contextLost = false,
    frame = 0,
    last = 0,
    time = 0;
  let renderedZoom = 0,
    visible = document.visibilityState !== "hidden",
    loaded = false,
    needsRender = true,
    readySent = false;
  let canvasLeft = 0,
    canvasTop = 0,
    canvasWidth = 1000;
  let lastFrame = "";
  const pointer = new THREE.Vector2();
  const parallax = new THREE.Vector2();
  const zeroPointer = new THREE.Vector2();
  const lastOrientation = new THREE.Quaternion();

  // Fetch the model, baked environment and shader sources in parallel. Any
  // asset failure returns the experience to its static preview.
  let environment;
  const assets = Promise.all([
    new GLTFLoader().loadAsync(MODEL_URL),
    loadSculptureShaders(),
    loadMetalEnvironment().then((texture) => {
      environment = texture;
      if (disposed) texture.dispose();
      return texture;
    }),
  ]);

  // Match the preview raster; CSS positions and scales this one square canvas.
  renderer.setPixelRatio(1);
  renderer.setSize(1000, 1000, false);
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 0.95;
  renderer.setClearColor(0x000000, 0);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.autoUpdate = false;
  renderer.shadowMap.type = THREE.PCFShadowMap;
  let canvas = renderer.domElement;
  const canvasStyle =
    "position:absolute;left:0;top:0;width:1000px;height:1000px;transform-origin:0 0";
  canvas.style.cssText = canvasStyle;
  mount.appendChild(canvas);
  const root = mount.closest(".sqlpage-world");
  delete root.dataset.renderer;
  const instructions = hitArea.querySelector("#view-instructions");
  const originalInstructions = instructions?.textContent;
  const gl = renderer.getContext();
  let fence,
    submittedAt = 0,
    samples = 0,
    slowFrames = 0,
    cpuBusyUntil = 0;
  let baked, context, lastTurn;
  const resetBudget = () => {
    if (fence) gl.deleteSync(fence);
    fence = undefined;
    samples = slowFrames = 0;
  };
  let timing;
  try {
    timing = new PerformanceObserver((list) => {
      for (const entry of list.getEntries())
        if (
          entry.scripts.reduce((total, script) => total + script.duration, 0) >=
          50
        )
          cpuBusyUntil = entry.startTime + entry.duration;
    });
    timing.observe({ type: "long-animation-frame" });
  } catch {
    /* Optional. */
  }
  const releaseGraphics = () => {
    renderer.domElement.removeEventListener("webglcontextlost", onLost);
    resetBudget();
    glow?.dispose();
    studioSet?.dispose();
    key.shadow.dispose();
    environment?.dispose();
    renderer.dispose();
    // This scene owns the context; losing it releases all its GPU allocations.
    renderer.forceContextLoss();
  };
  const useBakedViews = () => {
    releaseGraphics();
    baked = new Image();
    const replacement = document.createElement("canvas");
    replacement.width = 400;
    replacement.height = 400;
    context = replacement.getContext("2d");
    if (!context) {
      onError();
      return;
    }
    canvas.replaceWith(replacement);
    canvas = replacement;
    canvas.style.cssText = canvasStyle;
    root.dataset.renderer = "baked";
    root.dataset.scene = "loading";
    loaded = readySent = false;
    hitArea.inert = true;
    if (instructions)
      instructions.textContent =
        "Drag sideways or use left/right arrows to rotate. Use plus/minus to zoom and Home to reset. Swipe vertically or pinch to scroll or zoom the page.";
    baked.src = new URL(
      "../../assets/sculpture-turns.avif",
      import.meta.url,
    ).href;
    baked.decode().then(
      () => {
        loaded = !disposed;
        needsRender = true;
      },
      () => {
        if (!disposed) onError();
      },
    );
  };
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 70);
  cameraForSculpture(camera, CAMERA_DISTANCE);
  const liquidClock = { value: 0 };
  const interior = {
    uInteriorPosition: { value: new THREE.Vector3() },
    uInteriorRadius: { value: 1 },
    uInteriorPulse: { value: 2.4 },
  };
  const interiorLight = new THREE.PointLight(0x8bb8e8, 0.4, 2.1, 2);
  interiorLight.name = "Soft interior database light";
  const interiorScale = new THREE.Vector3();
  let glow, jewelReflection;
  const key = new THREE.DirectionalLight(0xe5edf1, 1.2);
  key.position.set(-3.5, 5, 5);
  key.castShadow = true;
  key.shadow.mapSize.set(1024, 1024);
  Object.assign(key.shadow.camera, {
    left: -3,
    right: 3,
    top: 3,
    bottom: -3,
    near: 0.5,
    far: 18,
  });
  key.shadow.normalBias = 0.012;
  key.shadow.radius = 8;
  scene.add(key);
  const rim = new THREE.DirectionalLight(0xa1bac7, 0.8);
  rim.position.set(4, 2, -2);
  scene.add(rim);
  const fill = new THREE.DirectionalLight(0x5c7987, 0.22);
  fill.position.set(-5, -0.5, -2);
  scene.add(fill);

  RectAreaLightUniformsLib.init();
  const softbox = new THREE.RectAreaLight(0xe5edf1, 14, 4, 7);
  softbox.position.set(-3, 2, 6);
  softbox.lookAt(0, 0, 0);
  scene.add(softbox);
  const edgeSoftbox = new THREE.RectAreaLight(0xa1bac7, 4, 1.2, 5);
  edgeSoftbox.position.set(3.5, 1.5, 2);
  edgeSoftbox.lookAt(0, 0, 0);
  scene.add(edgeSoftbox);
  scene.add(new THREE.HemisphereLight(0xa1bac7, 0x18262d, 0.55));

  const parallaxPivot = new THREE.Group();
  scene.add(parallaxPivot);
  const scrollPivot = new THREE.Group();
  parallaxPivot.add(scrollPivot);
  const dragPivot = new THREE.Group();
  scrollPivot.add(dragPivot);
  let diamond;
  let model;
  let wireMaterial;
  const modelCenter = new THREE.Vector3();
  let studioSet;
  assets
    .then(async ([gltf, shaders, studio]) => {
      if (disposed) return;
      glow = createSelectiveGlow(
        renderer,
        scene,
        camera,
        liquidClock,
        interior,
        shaders,
      );
      glow.setSize(1000, 1000, 1);
      jewelReflection = createFiniteJewelLight(JEWEL_EDGES, shaders);
      studioSet = createSculptureStudio(studio, shaders);
      scene.add(studioSet.group);
      performance.mark("sqlpage:assets-ready");
      const bounds = new THREE.Box3().setFromObject(gltf.scene);
      bounds.getCenter(modelCenter);
      const size = bounds.getSize(new THREE.Vector3());
      if (!Number.isFinite(size.y) || size.y <= 0)
        throw new Error("Invalid model bounds");
      gltf.scene.position.sub(modelCenter);
      model = gltf.scene;
      diamond = gltf.scene.getObjectByName("Diamond");
      gltf.scene.traverse((child) => {
        if (!(child instanceof THREE.Mesh)) return;
        for (const material of Array.isArray(child.material)
          ? child.material
          : [child.material]) {
          if (!(material instanceof THREE.MeshStandardMaterial)) continue;
          if (
            material.name.includes("Black") ||
            material.name.includes("Gunmetal")
          ) {
            finishMetal(material, studio, shaders);
            child.castShadow = true;
          }
          if (material.name.includes("Mirror Top"))
            jewelReflection.apply(material);
          if (material.name.includes("Separator"))
            finishLiquid(material, liquidClock, studio, interior, shaders);
          if (material.name.includes("Jewel Wire")) {
            material.color.set("#a9d2fa");
            material.emissive.set("#8bb8e8");
            material.emissiveIntensity = 1.6;
            wireMaterial = material;
          }
          if (material.name.includes("Endpoint")) {
            material.color.set("#c9e2fb");
            material.emissive.set("#a9d2fa");
            material.emissiveIntensity = 1.8;
          }
          if (material.name.includes("Facets")) {
            material.color.set("#8bb8e8");
            material.emissive.set("#466689");
            material.opacity = 0.035;
          }
          if (material.transparent) material.depthWrite = false;
        }
      });
      const jewelLight = new THREE.PointLight(0x8bb8e8, 0.15, 2.4, 2);
      jewelLight.position.set(0, 1.35, 0);
      gltf.scene.add(jewelLight);
      // Native model coordinates: stay inside the radius-1.04 database stack.
      interiorLight.position.set(0.4, -0.45, 0);
      gltf.scene.add(interiorLight);
      dragPivot.add(gltf.scene);
      await document.fonts.ready;
      if (disposed) return;
      // Use parallel shader compilation when supported. The loading state ends
      // only after a frame containing the sculpture and its glow is submitted.
      await renderer.compileAsync(scene, camera);
      if (disposed) return;
      loaded = true;
      needsRender = true;
    })
    .catch(() => {
      if (!disposed) onError();
    });

  const controls = createSculptureControls({
    mount,
    hitArea,
    camera,
    scrollPivot,
    pointer,
    isLoaded: () => loaded,
    yawOnly: () => !!baked,
  });
  const { pointers, dragQuaternion, zoom, reset } = controls;
  const onLost = (event) => {
    event.preventDefault();
    contextLost = true;
    cancelAnimationFrame(frame);
    onError();
  };
  renderer.domElement.addEventListener("webglcontextlost", onLost);
  const modelBounds = new THREE.Box3(),
    projection = new THREE.Matrix4();
  const screenBounds = { left: 0, right: 1, top: 0, bottom: 1 };
  const syncHitArea = () => {
    if (!model) return;
    // Box3 projects the same eight corners; avoid duplicating its min/max loop.
    if (baked) {
      modelBounds.min.set(-0.44, -0.62, 0);
      modelBounds.max.set(0.42, 0.56, 0);
    } else {
      modelBounds
        .setFromObject(model)
        .applyMatrix4(
          projection.multiplyMatrices(
            camera.projectionMatrix,
            camera.matrixWorldInverse,
          ),
        );
    }
    const { min, max } = modelBounds;
    screenBounds.left = (min.x + 1) * 500;
    screenBounds.right = (max.x + 1) * 500;
    screenBounds.top = (1 - max.y) * 500;
    screenBounds.bottom = (1 - min.y) * 500;
    if (pointers.size) return;
    const scale = canvasWidth / 1000;
    let { left, right, top, bottom } = screenBounds;
    left = canvasLeft + left * scale - 12;
    right = canvasLeft + right * scale + 12;
    top = canvasTop + top * scale - 12;
    bottom = canvasTop + bottom * scale + 12;
    // Clip in viewport space, then keep the target in the canvas host's local
    // coordinates. Rotation can otherwise put gesture centers off the screen.
    const origin = mount.getBoundingClientRect();
    left = clamp(left, -origin.left, innerWidth - origin.left);
    right = clamp(right, left, innerWidth - origin.left);
    top = clamp(top, 90 - origin.top, innerHeight - origin.top);
    bottom = clamp(bottom, top, innerHeight - origin.top);
    hitArea.style.left = `${left}px`;
    hitArea.style.top = `${top}px`;
    hitArea.style.width = `${right - left}px`;
    hitArea.style.height = `${bottom - top}px`;
  };
  const yaw = new THREE.Euler(0, 0, 0, "YXZ");
  const upAxis = new THREE.Vector3(0, 1, 0);
  const update = (timestamp) => {
    if (disposed || contextLost || !visible) return;
    const dt = Math.min((timestamp - last) / 1000 || 0.016, 0.05);
    last = timestamp;
    const state = getState();
    const ease = 1 - Math.exp(-dt * 8);
    renderedZoom = THREE.MathUtils.lerp(renderedZoom, controls.userZoom, ease);
    // Loading time and pointer movement must not advance the reference frame.
    if (!baked && loaded && readySent && state.motion) time += dt;
    parallax.lerp(
      !baked && readySent && state.motion ? pointer : zeroPointer,
      1 - Math.exp(-dt * 3.7),
    );
    controls.update(dt, readySent && state.motion);
    dragPivot.quaternion.copy(dragQuaternion);
    scrollPivot.quaternion
      .setFromAxisAngle(upAxis, state.turn ?? 0)
      .multiply(DEFAULT_TILT);
    parallaxPivot.rotation.set(parallax.y * 0.035, parallax.x * 0.055, 0);
    const distance = clamp(CAMERA_DISTANCE * Math.exp(renderedZoom), 5.9, 17);
    cameraForSculpture(camera, distance, parallax.x * 0.11, parallax.y * 0.075);
    // Both paths follow the same square frame and section anchors.
    const position = state.frame;
    canvasWidth = position.size * (baked ? Math.exp(-renderedZoom) : 1);
    canvasLeft = position.left + (position.size - canvasWidth) / 2;
    canvasTop = position.top + (position.size - canvasWidth) / 2;
    const frameKey = `${canvasLeft},${canvasTop},${canvasWidth},${state.turn}`;
    if (frameKey !== lastFrame) needsRender = true;
    lastFrame = frameKey;
    canvas.dataset.scrollTurn = String(state.turn ?? 0);
    canvas.style.transform = `translate3d(${canvasLeft}px,${canvasTop}px,0) scale(${canvasWidth / 1000})`;
    const changing =
      Math.abs(renderedZoom - controls.userZoom) > 0.0001 ||
      pointers.size > 0 ||
      !lastOrientation.equals(dragQuaternion) ||
      parallax.lengthSq() > 0.000001;
    const viewportTop = mount.getBoundingClientRect().top + canvasTop;
    const inView =
      state.opacity > 0.05 &&
      viewportTop + canvasWidth * 0.81 > 0 &&
      viewportTop + canvasWidth * 0.22 < window.innerHeight;
    if (!inView) resetBudget();
    if (readySent) {
      hitArea.inert = !inView;
      hitArea.tabIndex = inView ? 0 : -1;
    }
    // Poll even when no redraw is needed: idle time is not graphics latency.
    if (!baked && fence) {
      const pending = gl.clientWaitSync(fence, 0, 0) === gl.TIMEOUT_EXPIRED;
      const elapsed = performance.now() - submittedAt;
      if (pending && elapsed <= 1000) {
        frame = requestAnimationFrame(update);
        return;
      }
      gl.deleteSync(fence);
      fence = undefined;
      if (samples++ >= 2)
        slowFrames =
          elapsed > 34 && cpuBusyUntil < submittedAt ? slowFrames + 1 : 0;
      if (pending || slowFrames >= 4) {
        useBakedViews();
        frame = requestAnimationFrame(update);
        return;
      }
    }
    if (
      loaded &&
      (inView || !readySent) &&
      (state.motion || needsRender || changing)
    ) {
      if (baked) {
        syncHitArea();
        // 64 square views preserve the complete studio floor and its soft edges.
        const turn =
          (((state.turn ?? 0) +
            yaw.setFromQuaternion(dragQuaternion, "YXZ").y) /
            (2 * Math.PI)) *
          64;
        if (turn !== lastTurn) {
          const whole = Math.floor(turn),
            blend = turn - whole;
          context.clearRect(0, 0, 400, 400);
          for (let i = 0; i < 2; i++) {
            const index = (((whole + i) % 64) + 64) % 64;
            context.globalAlpha = i ? blend : 1 - blend;
            context.globalCompositeOperation = i ? "lighter" : "source-over";
            context.drawImage(
              baked,
              (index % 8) * 400,
              Math.floor(index / 8) * 400,
              400,
              400,
              0,
              0,
              400,
              400,
            );
          }
          lastTurn = turn;
        }
      } else {
        // Prepare GPU-only animation only when a new live frame can be drawn.
        liquidClock.value = time;
        if (diamond) {
          diamond.position.y = Math.sin(time * 0.62) * 0.012;
          diamond.rotation.y = Math.sin(time * 0.12) * 0.055;
        }
        interiorLight.position.set(
          Math.cos(time * ((Math.PI * 2) / 6)) * 0.4,
          -0.45 + Math.sin(time * ((Math.PI * 2) / 4)) * 0.28,
          Math.sin(time * ((Math.PI * 2) / 6)) * 0.4,
        );
        const pulse = 2.4 + Math.sin(time * ((Math.PI * 2) / 6)) * 0.3;
        interior.uInteriorPulse.value = pulse;
        interiorLight.intensity = (0.4 * pulse) / 2.4;
        scene.updateMatrixWorld(true);
        studioSet.update(model);
        scene.updateMatrixWorld(true);
        interiorLight.getWorldPosition(interior.uInteriorPosition.value);
        interiorLight.getWorldScale(interiorScale);
        interior.uInteriorRadius.value = Math.abs(interiorScale.x) * 1.15;
        if (diamond && wireMaterial)
          jewelReflection.update(diamond, camera, wireMaterial);
        syncHitArea();
        renderer.shadowMap.needsUpdate = true;
        renderer.render(scene, camera);
        glow.render(model, [studioSet.group]);
        fence = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
        submittedAt = performance.now();
      }
      needsRender = false;
      lastOrientation.copy(dragQuaternion);
      if (!readySent) {
        readySent = true;
        performance.mark("sqlpage:first-frame");
        performance.measure(
          "SQLPage scene startup",
          "sqlpage:scene-start",
          "sqlpage:first-frame",
        );
        onReady();
      }
    }
    frame = requestAnimationFrame(update);
  };
  const onVisibility = () => {
    visible = document.visibilityState !== "hidden";
    resetBudget();
    if (visible && !disposed && !contextLost) {
      last = performance.now();
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(update);
    } else cancelAnimationFrame(frame);
  };
  document.addEventListener("visibilitychange", onVisibility);
  frame = requestAnimationFrame(update);

  return {
    zoom,
    reset,
    dispose() {
      if (disposed) return;
      disposed = true;
      cancelAnimationFrame(frame);
      controls.dispose();
      document.removeEventListener("visibilitychange", onVisibility);
      timing?.disconnect();
      if (instructions) instructions.textContent = originalInstructions;
      if (baked) baked.src = "";
      if (!baked) releaseGraphics();
      canvas.remove();
    },
  };
}
