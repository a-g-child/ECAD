/** Draft artwork properties: one Apply commits the complete validated template edit. */
import type { TemplateGraphic, TemplateVariant } from './types';
import { element } from './ui';
import { textEditor } from './text';
import { TemplateRuns } from './template-runs';
export class TemplateArtwork {
  readonly root = element('section', 'template-artwork');
  readonly graphics: TemplateGraphic[];
  constructor(
    private model: TemplateVariant,
    private changed: () => void,
    private selectedId?: string,
  ) {
    this.graphics = structuredClone(model.graphics);
    this.render();
  }
  private field(
    parent: HTMLElement,
    name: string,
    value: string,
    set: (v: string) => void,
    multiline = false,
  ) {
    const input = multiline
      ? textEditor('property-input property-text', 3)
      : element('input', 'property-input');
    input.value = value;
    input.setAttribute('aria-label', name);
    const label = element('label', 'settings-field');
    label.append(element('span', '', name), input);
    parent.append(label);
    input.addEventListener('input', () => {
      set(input.value);
      this.changed();
    });
  }
  private render() {
    this.root.replaceChildren(element('h3', '', 'Template artwork'));
    if (!this.selectedId)
      this.root.append(
        element(
          'p',
          'inspector-note',
          'Paper coordinates in µm. Artwork may occupy margins. Apply properties commits changes.',
        ),
      );
    for (const kind of this.selectedId
      ? []
      : (['text', 'polyline', 'rectangle', 'ellipse'] as const)) {
      const b = element('button', 'plain', `Add ${kind === 'polyline' ? 'line / polyline' : kind}`);
      b.type = 'button';
      b.addEventListener('click', () => {
        const [pw, ph, left, , top] = this.model.paper.map(Number);
        const w = Math.min(40000, pw - left),
          h = Math.min(15000, ph - top);
        this.graphics.push({
          id: crypto.randomUUID().replaceAll('-', ''),
          kind,
          bounds: [left, top, w, h].map(String),
          contents: kind === 'text' ? [{ kind: 'literal', value: 'Text' }] : [],
          points:
            kind === 'polyline'
              ? [
                  ['0', '0'],
                  [String(w), String(h)],
                ]
              : [],
          font: 'sans-serif',
          text_height: '3500',
          stroke_width: '250',
          colour: '#283c46',
        });
        this.render();
        this.changed();
        this.root.querySelector('details:last-of-type')?.setAttribute('open', '');
      });
      this.root.append(b);
    }
    this.graphics.forEach((g, i) => {
      if (this.selectedId && g.id !== this.selectedId) return;
      const details = element('details');
      details.dataset.artworkId = g.id;
      details.open = !!this.selectedId;
      details.append(element('summary', '', `${i + 1} · ${g.kind}`));
      ['X', 'Y', 'Width', 'Height'].forEach((label, n) =>
        this.field(details, `Artwork ${label} (µm)`, g.bounds[n], (v) => (g.bounds[n] = v)),
      );
      this.field(details, 'Artwork colour', g.colour, (v) => (g.colour = v));
      if (g.kind === 'text') {
        this.field(details, 'Artwork font', g.font, (v) => (g.font = v));
        this.field(details, 'Artwork text height (µm)', g.text_height, (v) => (g.text_height = v));
        details.append(new TemplateRuns(g.contents, this.changed).runFields);
      } else {
        this.field(details, 'Artwork line width (µm)', g.stroke_width, (v) => (g.stroke_width = v));
      }
      if (g.kind === 'polyline') {
        this.field(
          details,
          'Local points (x,y per line)',
          g.points.map((p) => p.join(',')).join('\n'),
          (v) => {
            // Preserve invalid/unfinished input for Rust to reject at Apply, never drop a vertex.
            g.points = v.split(/\r\n?|\n/).map((line) => line.split(',').map((s) => s.trim()));
          },
          true,
        );
      }
      const remove = element('button', 'plain', 'Remove artwork');
      remove.type = 'button';
      remove.addEventListener('click', () => {
        this.graphics.splice(i, 1);
        this.render();
        this.changed();
      });
      details.append(remove);
      this.root.append(details);
    });
  }
}
