/** Exact micrometre paper presets; selecting one never scales drawing contents. */
import { element } from './ui';
const sizes: Record<string, [string, string, string]> = {
  A0: ['ISO A', '841000', '1189000'],
  A1: ['ISO A', '594000', '841000'],
  A2: ['ISO A', '420000', '594000'],
  A3: ['ISO A', '297000', '420000'],
  A4: ['ISO A', '210000', '297000'],
  A5: ['ISO A', '148000', '210000'],
  'Letter / ANSI A': ['US', '215900', '279400'],
  Legal: ['US', '215900', '355600'],
  'Tabloid / ANSI B': ['US', '279400', '431800'],
  'ANSI C': ['US', '431800', '558800'],
  'ANSI D': ['US', '558800', '863600'],
  'ANSI E': ['US', '863600', '1117600'],
};
export function paperPresets(
  parent: HTMLElement,
  paper: HTMLInputElement[],
  content: HTMLInputElement[],
  fit: HTMLInputElement,
  changed: () => void,
  advancedOpen: boolean,
  rememberAdvanced: (open: boolean) => void,
) {
  const controls = element('div');
  const select = (label: string, values: string[]) => {
    const input = element('select', 'property-input');
    input.setAttribute('aria-label', label);
    for (const value of values) {
      const option = element('option', '', value);
      option.value = value;
      input.append(option);
    }
    const row = element('label', 'settings-field');
    row.append(element('span', '', label), input);
    controls.append(row);
    return input;
  };
  const matching = Object.entries(sizes).find(
    ([, [, w, h]]) =>
      (paper[0].value === w && paper[1].value === h) ||
      (paper[0].value === h && paper[1].value === w),
  );
  const family = select('Paper family', ['ISO A', 'US', 'Custom']);
  family.value = matching?.[1][0] ?? 'Custom';
  const preset = select('Paper size', []);
  const orientation = select('Orientation', ['Portrait', 'Landscape']);
  orientation.value = BigInt(paper[0].value) > BigInt(paper[1].value) ? 'Landscape' : 'Portrait';
  const options = () => {
    preset.replaceChildren();
    for (const [key, [group]] of Object.entries(sizes))
      if (group === family.value) {
        const option = element('option', '', key);
        option.value = key;
        preset.append(option);
      }
    if (family.value === 'Custom') {
      const option = element('option', '', 'Custom');
      option.value = 'Custom';
      preset.append(option);
    }
  };
  options();
  if (matching) preset.value = matching[0];
  const advanced = element('details');
  advanced.append(element('summary', '', 'Advanced paper settings'));
  advanced.open = advancedOpen || !matching;
  for (const field of [...paper, ...content, fit]) advanced.append(field.closest('label')!);
  advanced.addEventListener('toggle', () => rememberAdvanced(advanced.open));
  parent.insertBefore(controls, parent.children[1] ?? null);
  parent.append(advanced);
  const apply = () => {
    const size = sizes[preset.value];
    if (size) {
      const [, w, h] = size;
      [paper[0].value, paper[1].value] = orientation.value === 'Landscape' ? [h, w] : [w, h];
      fit.checked = true;
      changed();
    }
  };
  family.addEventListener('change', () => {
    options();
    if (family.value === 'Custom') {
      advanced.open = true;
      return;
    }
    preset.value = family.value === 'ISO A' ? 'A4' : 'Letter / ANSI A';
    apply();
  });
  preset.addEventListener('change', apply);
  orientation.addEventListener('change', () => {
    if (family.value !== 'Custom') {
      apply();
      return;
    }
    const [w, h] = paper.map((p) => p.value);
    if (![w, h].every((value) => /^\d+$/.test(value))) {
      changed();
      return;
    }
    const landscape = BigInt(w) > BigInt(h);
    if (landscape !== (orientation.value === 'Landscape')) {
      [paper[0].value, paper[1].value] = [h, w];
      fit.checked = true;
      changed();
    }
  });
}
