import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";
import { GLTFLoader } from "https://cdn.jsdelivr.net/npm/three@0.186.1/examples/jsm/loaders/GLTFLoader.js/+esm";
import { createFiniteJewelLight } from "./finite-jewel-light.js";
import { JEWEL_EDGES } from "./jewel-edges.js";
import { layoutLandingFrame } from "./landing-frame.js";
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
import { createSelectiveGlow } from "./selective-glow.js";

const clamp = THREE.MathUtils.clamp;
/**
 * @typedef {{ motion: boolean, frame?: {left: number, top: number, size: number} }} SceneState
 * @typedef {{ zoom(amount: number): void, reset(): void, dispose(): void }} SceneController
 * @param {{ mount: HTMLElement, hitArea: HTMLElement, anchor: HTMLElement|null,
 *   getState: () => SceneState, onReady: () => void, onError: () => void }} options
 * @returns {SceneController}
 */
export function createDatabaseScene({
  mount,
  hitArea,
  anchor,
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
    width = 1,
    height = 1,
    visible = document.visibilityState !== "hidden",
    loaded = false,
    needsRender = true,
    readySent = false;
  let canvasLeft = 0,
    canvasTop = 0,
    canvasWidth = 1000,
    canvasHeight = 1000;
  let lastFrame = "";
  const pointer = new THREE.Vector2();
  const parallax = new THREE.Vector2();
  const zeroPointer = new THREE.Vector2();
  const lastOrientation = new THREE.Quaternion();
  const cleanup = [];
  const materials = new Set();
  const geometries = new Set();
  const textures = new Set();
  const trackResources = (object) => {
    object.traverse((child) => {
      if (
        child instanceof THREE.Mesh ||
        child instanceof THREE.Line ||
        child instanceof THREE.Points
      ) {
        geometries.add(child.geometry);
        for (const material of Array.isArray(child.material)
          ? child.material
          : [child.material]) {
          materials.add(material);
          for (const value of Object.values(material))
            if (value instanceof THREE.Texture) textures.add(value);
        }
      }
    });
  };
  const disposeObject = (object) => {
    object.traverse((child) => {
      if (child instanceof THREE.Mesh) {
        child.geometry.dispose();
        for (const material of Array.isArray(child.material)
          ? child.material
          : [child.material])
          material.dispose();
      }
    });
  };

  // Both URLs are preloaded by the HTML. Start consuming them together before
  // any scene setup; neither download waits for the other or a lighting bake.
  let environment;
  const assets = Promise.all([
    new GLTFLoader().loadAsync(MODEL_URL).then((gltf) => {
      // Track successful geometry even if the parallel lighting load fails.
      if (disposed) disposeObject(gltf.scene);
      else trackResources(gltf.scene);
      return gltf;
    }),
    loadMetalEnvironment().then((texture) => {
      environment = texture;
      if (disposed) texture.dispose();
      return texture;
    }),
  ]);

  renderer.setPixelRatio(
    Math.min(window.devicePixelRatio || 1, window.innerWidth < 680 ? 1.4 : 1.7),
  );
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 0.95;
  renderer.setClearColor(0x000000, 0);
  mount.appendChild(renderer.domElement);
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 70);
  cameraForSculpture(camera, CAMERA_DISTANCE);
  const liquidClock = { value: 0 };
  const interior = {
    uInteriorPosition: { value: new THREE.Vector3() },
    uInteriorRadius: { value: 1 },
    uInteriorPulse: { value: 14 },
  };
  const interiorLight = new THREE.PointLight(0x72c8de, 0.6, 2.1, 2);
  interiorLight.name = "Soft interior database light";
  const interiorScale = new THREE.Vector3();
  const glow = createSelectiveGlow(
    renderer,
    scene,
    camera,
    liquidClock,
    interior,
  );
  const jewelReflection = createFiniteJewelLight(JEWEL_EDGES);
  const key = new THREE.DirectionalLight(0xe5edf1, 1.7);
  key.position.set(-3.5, 5, 5);
  key.layers.enable(1);
  scene.add(key);
  const rim = new THREE.DirectionalLight(0xa1bac7, 0.8);
  rim.position.set(4, 2, -2);
  rim.layers.enable(1);
  scene.add(rim);
  const fill = new THREE.DirectionalLight(0x5c7987, 0.22);
  fill.position.set(-5, -0.5, -2);
  scene.add(fill);

  const placement = new THREE.Group();
  scene.add(placement);
  const parallaxPivot = new THREE.Group();
  placement.add(parallaxPivot);
  const scrollPivot = new THREE.Group();
  parallaxPivot.add(scrollPivot);
  const dragPivot = new THREE.Group();
  scrollPivot.add(dragPivot);
  const decoration = new THREE.Group();
  parallaxPivot.add(decoration);
  let diamond;
  let model;
  let wireMaterial;
  const modelCenter = new THREE.Vector3();
  let landing;
  const orbitalNodes = [];
  const orbMaterial = new THREE.MeshBasicMaterial({ color: 0x377e94 });
  const orbGeometry = new THREE.SphereGeometry(0.015, 10, 8);
  for (let i = 0; i < 4; i++) {
    const rx = 1.65 + i * 0.46,
      rz = 1.24 + i * 0.22,
      y = -1.3 - i * 0.18;
    const points = Array.from({ length: 192 }, (_, j) => {
      const a = (j / 192) * Math.PI * 2;
      return new THREE.Vector3(
        Math.cos(a) * rx,
        y + Math.sin(a) * (0.07 + i * 0.07),
        Math.sin(a) * rz,
      );
    });
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({
      color: i === 1 ? 0x3e7f90 : 0x22424f,
      transparent: true,
      opacity: i === 1 ? 0.24 : 0.14,
      depthWrite: false,
    });
    decoration.add(new THREE.LineLoop(geometry, material));
    for (let j = 0; j < 2; j++) {
      const orb = new THREE.Mesh(orbGeometry, orbMaterial);
      orb.scale.setScalar(j === 0 ? 1 : 0.66);
      decoration.add(orb);
      orbitalNodes.push({
        object: orb,
        rx,
        rz,
        y,
        phase: i * 1.34 + j * Math.PI,
        speed: 0.09 + i * 0.013,
      });
    }
  }

  const starsGeometry = new THREE.BufferGeometry();
  const positions = [],
    sizes = [],
    phases = [];
  let seed = 7331;
  const random = () => {
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return seed / 4294967296;
  };
  for (let i = 0; i < 65; i++) {
    positions.push(
      (random() - 0.5) * 8,
      (random() - 0.42) * 5.6,
      (random() - 0.5) * 5.5,
    );
    sizes.push(i % 11 === 0 ? 3 : 1 + random() * 1.4);
    phases.push(random() * Math.PI * 2);
  }
  starsGeometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(positions, 3),
  );
  starsGeometry.setAttribute(
    "pointSize",
    new THREE.Float32BufferAttribute(sizes, 1),
  );
  starsGeometry.setAttribute(
    "phase",
    new THREE.Float32BufferAttribute(phases, 1),
  );
  const starsMaterial = new THREE.ShaderMaterial({
    uniforms: {
      uTime: { value: 0 },
      uRatio: { value: renderer.getPixelRatio() },
    },
    vertexShader: `attribute float pointSize; attribute float phase; varying float brightness; uniform float uTime; uniform float uRatio;
      void main(){ vec3 p=position; p.y+=sin(uTime*.12+phase)*.06; vec4 mv=modelViewMatrix*vec4(p,1.0); gl_Position=projectionMatrix*mv; gl_PointSize=clamp(pointSize*uRatio*(7.0/-mv.z),1.0,12.0); brightness=.5+.35*sin(phase+uTime*.35); }`,
    fragmentShader: `varying float brightness; void main(){ float r=length(gl_PointCoord-.5); if(r>.5)discard; float a=pow(1.0-r*2.0,2.0)*brightness*.45; gl_FragColor=vec4(vec3(.13,.39,.48),a); }`,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const stars = new THREE.Points(starsGeometry, starsMaterial);
  decoration.add(stars);
  trackResources(decoration);
  assets
    .then(async ([gltf, studio]) => {
      if (disposed) return;
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
          )
            finishMetal(material, studio);
          if (material.name.includes("Mirror Top"))
            jewelReflection.apply(material);
          if (material.name.includes("Separator"))
            finishLiquid(material, liquidClock, studio, interior);
          if (material.name.includes("Jewel Wire")) {
            material.emissiveIntensity = 1.05;
            wireMaterial = material;
          }
          if (material.name.includes("Endpoint"))
            material.emissiveIntensity = 1.2;
          if (material.transparent) material.depthWrite = false;
        }
      });
      trackResources(gltf.scene);
      const jewelLight = new THREE.PointLight(0x72c8de, 0.09, 2.4, 2);
      jewelLight.position.set(0, 1.35, 0);
      gltf.scene.add(jewelLight);
      // Native model coordinates: stay inside the radius-1.04 database stack.
      interiorLight.position.set(0.4, -0.45, 0);
      gltf.scene.add(interiorLight);
      dragPivot.add(gltf.scene);
      await document.fonts.ready;
      if (disposed) return;
      resize();
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
  });
  const { pointers, dragQuaternion, zoom, reset } = controls;
  cleanup.push(() => controls.dispose());

  const resize = () => {
    width = Math.max(1, mount.clientWidth);
    landing = layoutLandingFrame(mount, anchor);
    height = landing.height;

    // Render the landing frame at the WebP's exact raster resolution. Both
    // surfaces then receive the same browser scaling, including on high-DPI phones.
    renderer.setPixelRatio(1);
    renderer.setSize(1000, 1000, false);
    glow.setSize(1000, 1000, 1);
    starsMaterial.uniforms.uRatio.value = 1;
    needsRender = true;
    window.dispatchEvent(new Event("sqlpage:layout"));
  };
  const observer = new ResizeObserver(resize);
  observer.observe(mount);
  resize();
  window.addEventListener("resize", resize);
  window.visualViewport?.addEventListener("resize", resize);
  cleanup.push(() => {
    window.removeEventListener("resize", resize);
    window.visualViewport?.removeEventListener("resize", resize);
  });
  document.fonts.ready.then(() => {
    if (!disposed) resize();
  });
  const onLost = (event) => {
    event.preventDefault();
    contextLost = true;
    cancelAnimationFrame(frame);
    onError();
  };
  renderer.domElement.addEventListener("webglcontextlost", onLost);
  const modelBounds = new THREE.Box3(),
    projectedCorner = new THREE.Vector3();
  const screenBounds = { left: 0, right: 1, top: 0, bottom: 1 };
  const syncHitArea = () => {
    if (!model) return;
    modelBounds.setFromObject(model);
    let left = Infinity,
      right = -Infinity,
      top = Infinity,
      bottom = -Infinity;
    for (let i = 0; i < 8; i++) {
      projectedCorner
        .set(
          i & 1 ? modelBounds.max.x : modelBounds.min.x,
          i & 2 ? modelBounds.max.y : modelBounds.min.y,
          i & 4 ? modelBounds.max.z : modelBounds.min.z,
        )
        .project(camera);
      const x = ((projectedCorner.x + 1) * renderer.domElement.width) / 2,
        y = ((1 - projectedCorner.y) * renderer.domElement.height) / 2;
      left = Math.min(left, x);
      right = Math.max(right, x);
      top = Math.min(top, y);
      bottom = Math.max(bottom, y);
    }
    screenBounds.left = left;
    screenBounds.right = right;
    screenBounds.top = top;
    screenBounds.bottom = bottom;
    if (pointers.size) return;
    left = canvasLeft + (left / renderer.domElement.width) * canvasWidth;
    right = canvasLeft + (right / renderer.domElement.width) * canvasWidth;
    top = canvasTop + (top / renderer.domElement.height) * canvasHeight;
    bottom = canvasTop + (bottom / renderer.domElement.height) * canvasHeight;
    left = clamp(left - 12, 0, width);
    right = clamp(right + 12, 0, width);
    top = clamp(top - 12, 90, height);
    bottom = clamp(bottom + 12, top, height);
    hitArea.style.left = `${left}px`;
    hitArea.style.top = `${top}px`;
    hitArea.style.width = `${right - left}px`;
    hitArea.style.height = `${bottom - top}px`;
  };
  const update = (timestamp) => {
    if (disposed || contextLost || !visible) return;
    const dt = Math.min((timestamp - last) / 1000 || 0.016, 0.05);
    last = timestamp;
    const state = getState();
    const ease = 1 - Math.exp(-dt * 8);
    renderedZoom = THREE.MathUtils.lerp(renderedZoom, controls.userZoom, ease);
    // Loading time and pointer movement must not advance the reference frame.
    if (loaded && readySent && state.motion) time += dt;
    parallax.lerp(
      readySent && state.motion ? pointer : zeroPointer,
      1 - Math.exp(-dt * 3.7),
    );
    controls.update(dt, readySent && state.motion);
    dragPivot.quaternion.copy(dragQuaternion);
    scrollPivot.quaternion.copy(DEFAULT_TILT);
    parallaxPivot.rotation.set(parallax.y * 0.035, parallax.x * 0.055, 0);
    const distance = clamp(CAMERA_DISTANCE * Math.exp(renderedZoom), 5.9, 17);
    cameraForSculpture(camera, distance, parallax.x * 0.11, parallax.y * 0.075);
    // One square renderer follows measured section anchors. The model, lighting,
    // camera and materials stay the same throughout the page.
    const position = state.frame ?? landing;
    canvasLeft = position.left;
    canvasTop = position.top;
    canvasWidth = canvasHeight = position.size;
    const frameKey = `${canvasLeft},${canvasTop},${canvasWidth}`;
    if (frameKey !== lastFrame) needsRender = true;
    lastFrame = frameKey;
    Object.assign(renderer.domElement.style, {
      position: "absolute",
      left: `${canvasLeft}px`,
      top: `${canvasTop}px`,
      width: `${canvasWidth}px`,
      height: `${canvasHeight}px`,
    });
    camera.aspect = 1;
    camera.updateProjectionMatrix();
    decoration.rotation.y = state.motion ? time * 0.014 : decoration.rotation.y;
    starsMaterial.uniforms.uTime.value = time;
    liquidClock.value = time;
    for (const orb of orbitalNodes) {
      const a = orb.phase + time * orb.speed;
      orb.object.position.set(
        Math.cos(a) * orb.rx,
        orb.y + Math.sin(a) * 0.07,
        Math.sin(a) * orb.rz,
      );
    }
    if (diamond) {
      diamond.position.y = Math.sin(time * 0.62) * 0.012;
      diamond.rotation.y = Math.sin(time * 0.12) * 0.055;
    }
    interiorLight.position.set(
      Math.cos(time * ((Math.PI * 2) / 6)) * 0.4,
      -0.45 + Math.sin(time * ((Math.PI * 2) / 4)) * 0.28,
      Math.sin(time * ((Math.PI * 2) / 6)) * 0.4,
    );
    const pulse = 14 + Math.sin(time * ((Math.PI * 2) / 3)) * 6;
    interior.uInteriorPulse.value = pulse;
    interiorLight.intensity = (0.6 * pulse) / 14;
    const changing =
      Math.abs(renderedZoom - controls.userZoom) > 0.0001 ||
      pointers.size > 0 ||
      !lastOrientation.equals(dragQuaternion) ||
      parallax.lengthSq() > 0.000001;
    const inView =
      canvasTop + canvasHeight * 0.81 > 0 &&
      canvasTop + canvasHeight * 0.22 < window.innerHeight;
    if (readySent) {
      hitArea.inert = !inView;
      hitArea.tabIndex = inView ? 0 : -1;
    }
    if (
      loaded &&
      (inView || !readySent) &&
      (state.motion || needsRender || changing)
    ) {
      scene.updateMatrixWorld(true);
      interiorLight.getWorldPosition(interior.uInteriorPosition.value);
      interiorLight.getWorldScale(interiorScale);
      interior.uInteriorRadius.value = Math.abs(interiorScale.x) * 1.15;
      syncHitArea();
      if (diamond && wireMaterial)
        jewelReflection.update(diamond, camera, wireMaterial);
      renderer.render(scene, camera);
      glow.render(model, [decoration], screenBounds);
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
      observer.disconnect();
      for (const fn of cleanup) fn();
      document.removeEventListener("visibilitychange", onVisibility);
      renderer.domElement.removeEventListener("webglcontextlost", onLost);
      for (const geometry of geometries) geometry.dispose();
      for (const material of materials) material.dispose();
      if (environment) textures.delete(environment);
      for (const texture of textures) texture.dispose();
      glow.dispose();
      environment?.dispose();
      renderer.dispose();
      renderer.forceContextLoss();
      renderer.domElement.remove();
    },
  };
}
