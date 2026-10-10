/** Trackball input is isolated from scene animation; vertical touch gestures stay native. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";

const clamp = THREE.MathUtils.clamp;

// A virtual trackball has no pole or Euler-angle limits; successive drags compose quaternions.
function trackballVector(x, y, rect) {
  const scale = Math.max(1, Math.min(rect.width, rect.height));
  const v = new THREE.Vector3(
    (2 * (x - rect.left) - rect.width) / scale,
    (rect.height - 2 * (y - rect.top)) / scale,
    0,
  );
  const length2 = v.x * v.x + v.y * v.y;
  v.z = length2 < 0.5 ? Math.sqrt(1 - length2) : 0.5 / Math.sqrt(length2);
  return v.normalize();
}

/** Own pointer capture, keyboard gestures, zoom, inertia, and their listeners. */
export function createSculptureControls({
  mount,
  hitArea,
  camera,
  scrollPivot,
  pointer,
  isLoaded,
}) {
  let userZoom = 0;
  const dragQuaternion = new THREE.Quaternion();
  const deltaQuaternion = new THREE.Quaternion();
  const localQuaternion = new THREE.Quaternion();
  const worldQuaternion = new THREE.Quaternion();
  const previousPoint = new THREE.Vector3();
  const currentPoint = new THREE.Vector3();
  const velocityAxis = new THREE.Vector3(0, 1, 0);
  let angularVelocity = 0,
    lastMove = 0;
  const pointers = new Map();
  let touchGesture = "pending";
  let touchStartX = 0,
    touchStartY = 0;
  const pointInParent = (x, y, out) => {
    out.copy(trackballVector(x, y, hitArea.getBoundingClientRect()));
    // Screen-space gestures are transformed into the rotating parent's coordinates.
    out.applyQuaternion(camera.quaternion);
    scrollPivot.getWorldQuaternion(worldQuaternion);
    out.applyQuaternion(worldQuaternion.invert());
    return out;
  };
  const onPointerDown = (event) => {
    if (!isLoaded() || (event.pointerType === "mouse" && event.button !== 0))
      return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    hitArea.setPointerCapture(event.pointerId);
    angularVelocity = 0;
    lastMove = performance.now();
    if (pointers.size === 1) {
      pointInParent(event.clientX, event.clientY, previousPoint);
      touchStartX = event.clientX;
      touchStartY = event.clientY;
      touchGesture = "pending";
    } else {
      // Multi-touch belongs to native page zoom, never the trackball.
      touchGesture = "scroll";
    }
  };
  const onPointerMove = (event) => {
    if (!pointers.has(event.pointerId)) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    if (pointers.size > 1) return;
    if (event.pointerType === "touch") {
      if (touchGesture === "pending") {
        const dx = Math.abs(event.clientX - touchStartX);
        const dy = Math.abs(event.clientY - touchStartY);
        // Leave taps and vertical intent alone. The browser takes over vertical
        // scrolling with pointercancel; a sideways start unlocks free rotation.
        if (Math.max(dx, dy) < 10) return;
        touchGesture = dx > dy * 1.25 ? "rotate" : "scroll";
      }
      if (touchGesture !== "rotate") return;
    }
    pointInParent(event.clientX, event.clientY, currentPoint);
    deltaQuaternion.setFromUnitVectors(previousPoint, currentPoint);
    dragQuaternion.premultiply(deltaQuaternion).normalize();
    const now = performance.now(),
      dt = Math.max((now - lastMove) / 1000, 0.01);
    const angle = 2 * Math.acos(clamp(deltaQuaternion.w, -1, 1));
    const sine = Math.sqrt(
      Math.max(0, 1 - deltaQuaternion.w * deltaQuaternion.w),
    );
    if (sine > 0.0001 && angle > 0.00001) {
      velocityAxis
        .set(
          deltaQuaternion.x / sine,
          deltaQuaternion.y / sine,
          deltaQuaternion.z / sine,
        )
        .normalize();
      angularVelocity = Math.min(angle / dt, 3.4);
    } else angularVelocity = 0;
    previousPoint.copy(currentPoint);
    lastMove = now;
  };
  const onPointerUp = (event) => {
    if (!pointers.delete(event.pointerId)) return;
    if (hitArea.hasPointerCapture(event.pointerId))
      hitArea.releasePointerCapture(event.pointerId);
    if (performance.now() - lastMove > 100 || event.type !== "pointerup")
      angularVelocity = 0;
    if (pointers.size === 1) {
      const p = [...pointers.values()][0];
      pointInParent(p.x, p.y, previousPoint);
      angularVelocity = 0;
    }
  };
  const zoom = (amount) => {
    if (Number.isFinite(amount))
      userZoom = clamp(userZoom + amount, -0.42, 0.4);
  };
  const reset = () => {
    dragQuaternion.identity();
    angularVelocity = 0;
    userZoom = 0;
    pointer.set(0, 0);
  };
  const onKey = (event) => {
    const axis = new THREE.Vector3();
    let angle = 0.17;
    if (event.key === "Home") {
      event.preventDefault();
      reset();
      return;
    }
    if (["+", "=", "-", "_"].includes(event.key)) {
      event.preventDefault();
      zoom(event.key === "-" || event.key === "_" ? 0.1 : -0.1);
      return;
    }
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      axis.set(0, event.shiftKey ? 0 : 1, event.shiftKey ? 1 : 0);
      if (event.key === "ArrowLeft") angle = -angle;
    } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
      axis.set(1, 0, 0);
      if (event.key === "ArrowUp") angle = -angle;
    } else return;
    event.preventDefault();
    angularVelocity = 0;
    dragQuaternion
      .premultiply(localQuaternion.setFromAxisAngle(axis, angle))
      .normalize();
  };
  const onParallax = (event) => {
    if (event.pointerType !== "mouse" || pointers.size) return;
    const rect = mount.getBoundingClientRect();
    pointer.set(
      clamp(
        ((event.clientX - rect.left) / Math.max(rect.width, 1)) * 2 - 1,
        -1,
        1,
      ),
      clamp(
        ((event.clientY - rect.top) / Math.max(rect.height, 1)) * 2 - 1,
        -1,
        1,
      ),
    );
  };
  const leave = () => pointer.set(0, 0);
  hitArea.addEventListener("pointerdown", onPointerDown);
  hitArea.addEventListener("pointermove", onPointerMove);
  hitArea.addEventListener("pointerup", onPointerUp);
  hitArea.addEventListener("pointercancel", onPointerUp);
  hitArea.addEventListener("lostpointercapture", onPointerUp);
  hitArea.addEventListener("keydown", onKey);
  window.addEventListener("pointermove", onParallax, { passive: true });
  document.documentElement.addEventListener("pointerleave", leave);
  const dispose = () => {
    hitArea.removeEventListener("pointerdown", onPointerDown);
    hitArea.removeEventListener("pointermove", onPointerMove);
    hitArea.removeEventListener("pointerup", onPointerUp);
    hitArea.removeEventListener("pointercancel", onPointerUp);
    hitArea.removeEventListener("lostpointercapture", onPointerUp);
    hitArea.removeEventListener("keydown", onKey);
    window.removeEventListener("pointermove", onParallax);
    document.documentElement.removeEventListener("pointerleave", leave);
  };

  return {
    pointers,
    dragQuaternion,
    zoom,
    reset,
    get userZoom() {
      return userZoom;
    },
    update(dt, motion) {
      if (pointers.size === 0 && angularVelocity > 0.003 && motion) {
        dragQuaternion
          .premultiply(
            localQuaternion.setFromAxisAngle(
              velocityAxis,
              angularVelocity * dt,
            ),
          )
          .normalize();
        angularVelocity *= Math.exp(-dt * 4.3);
      }
    },
    dispose() {
      for (const id of pointers.keys()) {
        if (hitArea.hasPointerCapture(id)) hitArea.releasePointerCapture(id);
      }
      pointers.clear();
      dispose();
    },
  };
}
