/** Shared camera calibration for the static preview and live square renderer. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";

export const DEFAULT_TILT = new THREE.Quaternion().setFromEuler(
  new THREE.Euler(0.39, -0.24, -0.245, "XYZ"),
);
export const CAMERA_DISTANCE = 9.6;

export function cameraForSculpture(camera, distance, x = 0, y = 0) {
  camera.position.set(x, distance * 0.22 + y, distance);
  camera.lookAt(0, 0, 0);
  camera.updateMatrixWorld();
}
