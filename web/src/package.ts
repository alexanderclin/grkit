import { effectUrl, siteUrl, type Effect } from './catalog';
import wasmUrl from './wasm/grkit_wasm_bg.wasm?url';

/** A request fails when no data arrives for this long, so a stalled connection ends in an error. */
const STALL_MS = 20_000;

/** One file that the package needs. A download keeps its bytes for later packages. */
interface Transfer {
  bytes: Promise<Uint8Array>;
  received: number;
}

const transfers = new Map<string, Transfer>();
/** Called on each received chunk, so a running build can report its progress. */
const watchers = new Set<() => void>();

function notify() {
  for (const watcher of watchers) watcher();
}

async function download(url: string, transfer: Transfer): Promise<Uint8Array> {
  const controller = new AbortController();
  let timer: ReturnType<typeof setTimeout> | undefined;
  const arm = () => {
    clearTimeout(timer);
    timer = setTimeout(() => controller.abort(new Error(`No data for ${STALL_MS} ms: ${url}`)), STALL_MS);
  };
  arm();
  try {
    const response = await fetch(url, { cache: 'no-cache', signal: controller.signal });
    if (!response.ok) {
      throw new Error(`Request failed: ${response.status} ${url}`);
    }
    const chunks: Uint8Array[] = [];
    const reader = response.body!.getReader();
    for (;;) {
      arm();
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
      transfer.received += value.length;
      notify();
    }
    const bytes = new Uint8Array(transfer.received);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.length;
    }
    return bytes;
  } finally {
    clearTimeout(timer);
  }
}

/** The download of `url`: a new one, or the one in progress or done. A failed one starts again. */
function transfer(url: string): Transfer {
  let item = transfers.get(url);
  if (!item) {
    const created: Transfer = { bytes: Promise.resolve(new Uint8Array()), received: 0 };
    created.bytes = download(url, created).catch((error: unknown) => {
      if (transfers.get(url) === created) transfers.delete(url);
      throw error;
    });
    transfers.set(url, created);
    item = created;
  }
  return item;
}

type Wasm = typeof import('./wasm/grkit_wasm.js');
let wasm: Promise<Wasm> | undefined;

/** The packager, loaded once. Its module downloads like the other files, with a stall limit. */
function loadWasm(): Promise<Wasm> {
  wasm ??= (async () => {
    const [module, bytes] = await Promise.all([
      import('./wasm/grkit_wasm.js'),
      transfer(wasmUrl).bytes,
    ]);
    await module.default({ module_or_path: bytes });
    return module;
  })().catch((error: unknown) => {
    wasm = undefined;
    throw error;
  });
  return wasm;
}

const buildUrl = (target: string) => siteUrl(`runtime/${target}/build.json`);
const guideUrl = (camera: string, language: string) => siteUrl(`guides/${camera}/${language}.pdf`);
const runtimeUrl = (target: string, path: string) => siteUrl(`runtime/${target}/sd/${path}`);

/**
 * The runtime paths that `build.json` lists, read without the packager so that the files can
 * download while the packager loads. The packager checks each file against the manifest.
 */
function listedPaths(build: Uint8Array): string[] {
  try {
    const files: unknown = JSON.parse(new TextDecoder().decode(build)).files;
    return Array.isArray(files)
      ? files.flatMap((file) => (typeof file?.path === 'string' ? [file.path as string] : []))
      : [];
  } catch {
    return [];
  }
}

/**
 * Starts the downloads of a package for `camera` before the click on Download: the packager, the
 * runtime, and the guide. A later `buildPackage` uses them. Errors stay silent here, and the
 * build requests a failed file again.
 */
export function prefetchPackage(target: string, camera: string, language: string): void {
  const ignore = () => {};
  loadWasm().catch(ignore);
  transfer(guideUrl(camera, language)).bytes.catch(ignore);
  transfer(buildUrl(target))
    .bytes.then((build) => {
      for (const path of listedPaths(build)) {
        transfer(runtimeUrl(target, path)).bytes.catch(ignore);
      }
    })
    .catch(ignore);
}

async function sha256(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes as Uint8Array<ArrayBuffer>);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('');
}

/** One user-supplied LUT that the build fits into the package as a custom effect. */
export interface CustomLut {
  /** The `.cube` file bytes. */
  bytes: Uint8Array;
  /** The file name, recorded in the effect's attribution. */
  source: string;
  id: string;
  name: string;
  /** The ADJ icon text: one to four characters. */
  label: string;
  /** The icon shape: `film`, `plain`, or `filter`. */
  shape: string;
  /** The encoding the LUT expects: `srgb`, `linear`, or `vlog`. */
  input: string;
}

/** The fit scores that `Package.importLut` reports for one LUT. */
export interface LutFitScore {
  mean_de: number;
  max_de: number;
  standard_mean_de: number;
}

/**
 * Fits one custom LUT against the target's runtime, without building a package,
 * so the page can show the fit score before Download. Throws when the LUT
 * cannot be fitted for this target.
 */
export async function fitCustomLut(target: string, lut: CustomLut): Promise<LutFitScore> {
  const [{ Package }, build] = await Promise.all([loadWasm(), transfer(buildUrl(target)).bytes]);
  const builder = new Package(build);
  try {
    const json = builder.importLut(
      lut.bytes,
      lut.source,
      lut.id,
      lut.name,
      lut.label,
      lut.shape,
      lut.input,
    );
    return JSON.parse(json) as LutFitScore;
  } finally {
    builder.free();
  }
}

/**
 * Builds GRKit.zip in the browser from the target's published runtime and the selected effects.
 * `camera` selects the ModeSet file. `omitted` names the plugins that the package does not install.
 * `language` selects the installation guide, which the site publishes as `guides/<camera>/<language>.pdf`.
 * `customLuts` are fitted in the browser and added as effects alongside the selected ones.
 * grkit-wasm checks each runtime file against `build.json` and each effect's format.
 * `onProgress` receives the number of bytes of the files of this package that have arrived.
 */
export async function buildPackage(
  target: string,
  camera: string,
  effects: readonly Effect[],
  omitted: readonly string[],
  language: string,
  customLuts: readonly CustomLut[],
  onProgress: (received: number) => void,
): Promise<Blob> {
  const used = new Set<string>([wasmUrl]);
  const use = (url: string) => {
    used.add(url);
    return transfer(url).bytes;
  };
  const report = () => {
    let received = 0;
    for (const url of used) received += transfers.get(url)?.received ?? 0;
    onProgress(received);
  };
  watchers.add(report);
  try {
    // The packager, the runtime files, the guide, and the effects download at the same time.
    const ignore = () => {};
    use(buildUrl(target))
      .then((build) => {
        for (const path of listedPaths(build)) use(runtimeUrl(target, path)).catch(ignore);
      })
      .catch(ignore);
    const effectBytes = Promise.all(effects.map((effect) => use(effectUrl(target, effect.file))));
    // The build awaits the effects later. Until then, a failure is not an unhandled rejection.
    effectBytes.catch(ignore);
    const [{ Package }, build, guide] = await Promise.all([
      loadWasm(),
      use(buildUrl(target)),
      use(guideUrl(camera, language)),
    ]);
    report();
    const builder = new Package(build);
    try {
      builder.setCamera(camera);
      builder.setLanguage(language);
      builder.setGuide(guide);
      const paths = builder.runtimePaths();
      const files = await Promise.all(paths.map((path) => use(runtimeUrl(target, path))));
      for (const [index, bytes] of files.entries()) {
        builder.addRuntimeFile(paths[index]!, bytes);
      }
      for (const [index, blob] of (await effectBytes).entries()) {
        const effect = effects[index]!;
        if ((await sha256(blob)) !== effect.sha256) {
          throw new Error(`Effect checksum mismatch: ${effect.id}`);
        }
        builder.addEffect(blob);
      }
      for (const lut of customLuts) {
        builder.importLut(lut.bytes, lut.source, lut.id, lut.name, lut.label, lut.shape, lut.input);
      }
      for (const name of omitted) {
        builder.omitPlugin(name);
      }
      const zip = builder.build();
      return new Blob([zip as Uint8Array<ArrayBuffer>], { type: 'application/zip' });
    } finally {
      builder.free();
    }
  } catch (error) {
    // A file can be stale after a deployment, so a new attempt downloads them all again.
    for (const url of used) {
      if (url !== wasmUrl) transfers.delete(url);
    }
    throw error;
  } finally {
    watchers.delete(report);
  }
}

/** Starts the download of a blob as a file. */
/** The ZIP that turns SD scripts off on `camera`: its ModeSet file with the disable command. */
export async function buildScriptsOff(camera: string): Promise<Blob> {
  const module = await loadWasm();
  return new Blob([module.scriptsOffPackage(camera) as Uint8Array<ArrayBuffer>], {
    type: 'application/zip',
  });
}

export function save(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  // Some browsers read the URL after the click returns.
  setTimeout(() => URL.revokeObjectURL(url), 60_000);
}
