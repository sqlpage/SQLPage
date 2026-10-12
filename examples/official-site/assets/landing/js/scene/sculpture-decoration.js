/** Deterministic stars and orbital tracks; resources are owned by the scene. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";

export function createSculptureDecoration(shaders) {
  const decoration = new THREE.Group();
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
    },
    vertexShader: shaders.starsVertex,
    fragmentShader: shaders.starsFragment,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
  const stars = new THREE.Points(starsGeometry, starsMaterial);
  decoration.add(stars);
  return {
    group: decoration,
    update(time, motion) {
      if (motion) decoration.rotation.y = time * 0.014;
      starsMaterial.uniforms.uTime.value = time;
      for (const orb of orbitalNodes) {
        const a = orb.phase + time * orb.speed;
        orb.object.position.set(
          Math.cos(a) * orb.rx,
          orb.y + Math.sin(a) * 0.07,
          Math.sin(a) * orb.rz,
        );
      }
    },
  };
}
