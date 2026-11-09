/** Adapt union-inspected drawing fonts to the common palette; commands retain captured targets. */
import type { PropertyRow } from './types';
import { FontPalette } from './font-palette';
import type { FontProperties, FontProperty } from './font-properties';
import { writeDimension } from './dimensions';
const keys = { font: 'font_family', height: 'text_height' } as const;
export function schematicFonts(
  rows: PropertyRow[],
  commit: (changes: Record<string, string | null>) => Promise<void>,
) {
  const font = rows.find((r) => r.key === keys.font && r.editable);
  if (!font) return null;
  const supported = rows.filter(
    (r) => Object.values(keys).includes(r.key as typeof keys.font) && r.editable,
  );
  const style: FontProperties = {};
  const mixed: FontProperty[] = [];
  const originals = new Map<string, string>();
  for (const [key, id] of Object.entries(keys)) {
    const row = supported.find((r) => r.key === id);
    if (!row) continue;
    (style as Record<string, unknown>)[key] = row.value ?? '';
    if (row.mixed) mixed.push(key as FontProperty);
  }
  const pending: Record<string, string | null> = {};
  const finish = async () => {
    if (!Object.keys(pending).length) return;
    await commit({ ...pending });
  };
  const palette = new FontPalette(
    [style],
    'Text',
    (key) => {
      if (key !== 'font' && key !== 'height') return;
      pending[keys[key]] = style[key] ?? null;
    },
    () => void finish(),
    {
      mixed,
      labels: { font: font.label, height: supported.find((r) => r.key === keys.height)?.label },
      field: (key, input) => {
        if (key !== 'font' && key !== 'height') return;
        const row = supported.find((r) => r.key === keys[key])!;
        input.dataset.propertyKey = row.key;
        originals.set(row.key, row.value ?? '');
        input.title = `${row.origin ?? ''}\n${row.applicable} applicable · ${row.unsupported} unsupported · Enter applies`;
      },
    },
  );
  for (const input of palette.root.querySelectorAll<HTMLInputElement>('[data-property-key]')) {
    const host = input.closest('.font-palette-size,.font-palette-family') as HTMLElement;
    host.dataset.property = input.dataset.propertyKey;
  }
  palette.root.addEventListener('keydown', (e) => {
    if (e.isComposing) return;
    if (
      e.key === 'Enter' &&
      (!(e.target instanceof HTMLTextAreaElement) || e.ctrlKey || e.metaKey)
    ) {
      e.preventDefault();
      e.stopPropagation();
      void finish();
    }
    if (e.key === 'Escape') {
      e.stopPropagation();
      for (const input of palette.root.querySelectorAll<HTMLInputElement>('[data-property-key]')) {
        const key = input.dataset.propertyKey!;
        writeDimension(input, originals.get(key)!);
        input.dispatchEvent(new Event('input'));
      }
      for (const key of Object.keys(pending)) delete pending[key];
      (e.target as HTMLElement).blur();
    }
  });
  palette.root.addEventListener('change', (e) => {
    if (e.target instanceof HTMLSelectElement) void finish();
  });
  return { root: palette.root, keys: supported.map((r) => r.key) };
}
