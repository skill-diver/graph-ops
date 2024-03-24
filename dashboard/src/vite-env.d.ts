/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_OFNIL_BACKEND_URL: string;
  readonly VITE_PORT: number;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
