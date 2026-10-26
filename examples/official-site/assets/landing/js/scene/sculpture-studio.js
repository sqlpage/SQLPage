/** A matte studio sweep catches real light and the sculpture's shadow. */
import * as THREE from "https://cdn.jsdelivr.net/npm/three@0.186.1/+esm";

export function createSculptureStudio() {
  const group = new THREE.Group();
  const profile = [
    [5, 0],
    [-1.8, 0],
  ];
  for (let i = 1; i <= 20; i++) {
    const angle = ((i / 20) * Math.PI) / 2;
    profile.push([-1.8 - Math.sin(angle) * 1.2, 1.2 - Math.cos(angle) * 1.2]);
  }
  profile.push([-3, 7]);
  const positions = [],
    indices = [];
  profile.forEach(([z, y], i) => {
    positions.push(-6, y, z, 6, y, z);
    if (i) {
      const n = i * 2;
      indices.push(n - 2, n - 1, n, n - 1, n + 1, n);
    }
  });
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute(
    "position",
    new THREE.Float32BufferAttribute(positions, 3),
  );
  geometry.setIndex(indices);
  geometry.computeVertexNormals();
  const theme = getComputedStyle(document.documentElement);
  const material = new THREE.MeshStandardMaterial({
    color: theme.getPropertyValue("--tblr-bg-surface-tertiary").trim(),
    roughness: 0.86,
    metalness: 0.12,
    transparent: true,
    depthWrite: false,
    opacity: 0.65,
  });
  material.onBeforeCompile = (shader) => {
    shader.vertexShader = shader.vertexShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vStudioPosition;",
      )
      .replace(
        "#include <begin_vertex>",
        "#include <begin_vertex>\nvStudioPosition = position;",
      );
    shader.fragmentShader = shader.fragmentShader
      .replace(
        "#include <common>",
        "#include <common>\nvarying vec3 vStudioPosition;",
      )
      .replace(
        "#include <alphamap_fragment>",
        `#include <alphamap_fragment>
        float edge = 1.0 - smoothstep(1.2, 2.8, abs(vStudioPosition.x));
        float wall = 1.0 - smoothstep(1.8, 5.0, vStudioPosition.y);
        float floorEdge = 1.0 - smoothstep(1.0, 3.8, vStudioPosition.z);
        float grain = fract(sin(dot(floor(vStudioPosition.xz * 800.0), vec2(12.9898, 78.233))) * 43758.5453);
        diffuseColor.rgb *= 0.94 + grain * 0.12;
        diffuseColor.a *= edge * wall * floorEdge;
      `,
      );
  };
  material.customProgramCacheKey = () => "sqlpage-studio-sweep-v1";
  const sweep = new THREE.Mesh(geometry, material);
  sweep.receiveShadow = true;
  group.add(sweep);
  const spill = new THREE.PointLight(
    theme.getPropertyValue("--tblr-primary").trim(),
    2.4,
    5,
    2,
  );
  spill.position.set(0.3, 0.85, -0.6);
  group.add(spill);
  const bounds = new THREE.Box3();
  return {
    group,
    dispose() {
      geometry.dispose();
      material.dispose();
      group.removeFromParent();
    },
    update(model) {
      // Follow the lowest point of the tilted/dragged model without rotating
      // the room with it. The shadow remains attached to the contact surface.
      bounds.setFromObject(model);
      group.position.y = bounds.min.y - 0.025;
    },
  };
}
