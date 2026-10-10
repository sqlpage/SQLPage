// Resolve assets relative to this module, including when hosted in a subdirectory.
export const MODEL_URL = new URL(
  "../../assets/models/sqlpage-database-bbca200908a1.glb",
  import.meta.url,
).href;
export const STUDIO_URL = new URL(
  "../../assets/lighting/sqlpage-studio-cubeuv-0d7daea56fd0.png",
  import.meta.url,
).href;
export const STUDIO_RADIANCE_SCALE = 1.5;
