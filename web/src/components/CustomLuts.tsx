import { useEffect, useRef, useState, type ChangeEvent } from 'react';
import type { Strings } from '../i18n';
import { fitCustomLut, type CustomLut, type LutFitScore } from '../package';

/** One uploaded LUT and the fields the package build needs, plus a stable key. */
export interface CustomLutEntry extends CustomLut {
  key: string;
}

type FitState =
  | { state: 'checking' }
  | { state: 'ok'; score: LutFitScore }
  | { state: 'error' };

const SHAPES = ['film', 'plain', 'filter'] as const;
const INPUTS = ['srgb', 'linear', 'vlog'] as const;

/** An effect ID from a file name: `lut-` plus lowercase words joined by hyphens. */
function deriveId(fileName: string, taken: ReadonlySet<string>): string {
  const base = fileName.replace(/\.cube$/i, '');
  const slug = base
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  const stem = `lut-${slug || 'custom'}`.slice(0, 31).replace(/-+$/, '');
  let id = stem;
  for (let n = 2; taken.has(id); n += 1) id = `${stem.slice(0, 28)}-${n}`;
  return id;
}

/** The default display name: the file name without `.cube`, fitted to the 16-character limit. */
function deriveName(fileName: string): string {
  const base = fileName.replace(/\.cube$/i, '').replace(/[_-]+/g, ' ').trim();
  return (base || 'Custom LUT').slice(0, 16);
}

/** The default icon label: the first alphanumeric characters of the file name, uppercased. */
function deriveLabel(fileName: string): string {
  const letters = fileName.replace(/\.cube$/i, '').toUpperCase().replace(/[^A-Z0-9]/g, '');
  return (letters || 'LUT').slice(0, 4);
}

interface Props {
  strings: Strings;
  /** The runtime target the package is built for; the fit runs against it. */
  target: string;
  /** Effect IDs the package already takes (the selected catalog effects). */
  takenIds: ReadonlySet<string>;
  /** How many more effects the package can hold. */
  room: number;
  luts: readonly CustomLutEntry[];
  onChange: (luts: CustomLutEntry[]) => void;
}

export function CustomLuts({ strings: t, target, takenIds, room, luts, onChange }: Props) {
  const fileInput = useRef<HTMLInputElement>(null);
  const [fits, setFits] = useState<ReadonlyMap<string, FitState>>(new Map());
  const checking = useRef(new Set<string>());

  // Fit each LUT once per target and input encoding, so the score can show
  // before Download. Name, label, and shape do not change the fit.
  useEffect(() => {
    for (const lut of luts) {
      const key = `${target}|${lut.key}|${lut.input}`;
      if (fits.has(key) || checking.current.has(key)) continue;
      checking.current.add(key);
      setFits((prev) => new Map(prev).set(key, { state: 'checking' }));
      fitCustomLut(target, lut)
        .then((score) => setFits((prev) => new Map(prev).set(key, { state: 'ok', score })))
        .catch(() => setFits((prev) => new Map(prev).set(key, { state: 'error' })))
        .finally(() => checking.current.delete(key));
    }
  }, [fits, luts, target]);

  async function addFiles(event: ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? []);
    event.target.value = '';
    const taken = new Set([...takenIds, ...luts.map((lut) => lut.id)]);
    const added: CustomLutEntry[] = [];
    for (const file of files.slice(0, Math.max(0, room - luts.length))) {
      const id = deriveId(file.name, taken);
      taken.add(id);
      added.push({
        key: `${id}-${file.size}-${file.lastModified}`,
        bytes: new Uint8Array(await file.arrayBuffer()),
        source: file.name,
        id,
        name: deriveName(file.name),
        label: deriveLabel(file.name),
        shape: 'film',
        input: 'srgb',
      });
    }
    if (added.length > 0) onChange([...luts, ...added]);
  }

  function update(key: string, patch: Partial<CustomLutEntry>) {
    onChange(luts.map((lut) => (lut.key === key ? { ...lut, ...patch } : lut)));
  }

  return (
    <div className="custom-luts">
      <h3>{t.luts.heading}</h3>
      <p className="note">{t.luts.note}</p>
      {luts.map((lut) => {
        const fit = fits.get(`${target}|${lut.key}|${lut.input}`);
        return (
          <div key={lut.key} className="custom-lut">
            <span className="custom-lut-file">{lut.source}</span>
            <label>
              {t.luts.name}
              <input
                type="text"
                value={lut.name}
                maxLength={16}
                onChange={(event) =>
                  update(lut.key, {
                    name: event.target.value.replace(/[^\x20-\x7E]/g, ''),
                  })
                }
              />
            </label>
            <label>
              {t.luts.label}
              <input
                type="text"
                value={lut.label}
                maxLength={4}
                onChange={(event) =>
                  update(lut.key, {
                    label: event.target.value.toUpperCase().replace(/[^A-Z0-9]/g, ''),
                  })
                }
              />
            </label>
            <label>
              {t.luts.shape}
              <select
                value={lut.shape}
                onChange={(event) => update(lut.key, { shape: event.target.value })}
              >
                {SHAPES.map((shape) => (
                  <option key={shape} value={shape}>
                    {t.luts.shapes[shape]}
                  </option>
                ))}
              </select>
            </label>
            <label>
              {t.luts.input}
              <select
                value={lut.input}
                onChange={(event) => update(lut.key, { input: event.target.value })}
              >
                {INPUTS.map((input) => (
                  <option key={input} value={input}>
                    {t.luts.inputs[input]}
                  </option>
                ))}
              </select>
            </label>
            <span className="custom-lut-fit" aria-live="polite">
              {fit?.state === 'checking' && t.luts.checking}
              {fit?.state === 'ok' &&
                t.luts.fit(fit.score.mean_de.toFixed(2), fit.score.max_de.toFixed(2))}
              {fit?.state === 'error' && <span className="error">{t.luts.error}</span>}
            </span>
            <button
              type="button"
              className="button"
              onClick={() => onChange(luts.filter((item) => item.key !== lut.key))}
            >
              {t.luts.remove}
            </button>
          </div>
        );
      })}
      <input
        ref={fileInput}
        type="file"
        accept=".cube"
        multiple
        hidden
        onChange={addFiles}
      />
      <button
        type="button"
        className="button"
        disabled={luts.length >= room}
        onClick={() => fileInput.current?.click()}
      >
        {t.luts.add}
      </button>
    </div>
  );
}
