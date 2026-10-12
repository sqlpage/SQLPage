/** Native ES modules fetch GLSL beside the scene; no runtime build step is needed. */
const files = {
  blurFragment: "blur-fragment.glsl",
  compositeFragment: "composite-fragment.glsl",
  compositeVertex: "composite-vertex.glsl",
  emissionFragment: "emission-fragment.glsl",
  emissionVertex: "emission-vertex.glsl",
  fullscreenVertex: "fullscreen-vertex.glsl",
  interiorLight: "interior-light.glsl",
  jewelLight: "jewel-light.glsl",
  jewelReflection: "jewel-reflection.glsl",
  liquidFinish: "liquid-finish.glsl",
  liquidFlow: "liquid-flow.glsl",
  metalFinish: "metal-finish.glsl",
  starsFragment: "stars-fragment.glsl",
  starsVertex: "stars-vertex.glsl",
};
let sources;

/** Share successful loads across retries, but never cache a failed network attempt. */
export function loadSculptureShaders() {
  sources ??= Promise.all(
    Object.entries(files).map(async ([name, file]) => {
      const url = new URL(`./shaders/${file}`, import.meta.url);
      const response = await fetch(url);
      if (!response.ok)
        throw new Error(`Unable to load sculpture shader: ${file}`);
      const source = await response.text();
      if (!source.trim()) throw new Error(`Empty sculpture shader: ${file}`);
      return [name, source];
    }),
  )
    .then(Object.fromEntries)
    .catch((error) => {
      sources = undefined;
      throw error;
    });
  return sources;
}
