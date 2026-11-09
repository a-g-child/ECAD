import { controlHelp } from './control-help';
import { PictureFields } from './picture-controls';
/** Draft artwork properties: one Apply commits the complete validated template edit. */
import { dimensionPoints, dimension, readDimension } from './dimensions';
import type { TemplateGraphic, TemplateVariant } from './types';
import { element } from './ui';
import { appearanceControl, type AppearanceKind } from './appearance-controls';
import { textEditor } from './text';
import { FontPalette } from './font-palette';
import { TemplateRuns } from './template-runs';
export class TemplateArtwork {
  readonly root = element('section', 'template-artwork');
  readonly graphics: TemplateGraphic[];
  constructor(
    private model: TemplateVariant,
    private changed: () => void,
    private selectedId?: string,
    private complete: () => void = () => {},
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
    appearance?: AppearanceKind,
  ) {
    const input = multiline
      ? textEditor('property-input property-text', 3)
      : element('input', 'property-input');
    input.value = value;
    input.setAttribute('aria-label', name);
    const label = element('label', 'settings-field');
    label.append(
      element('span', '', name),
      appearance ? appearanceControl(input, appearance) : input,
    );
    parent.append(label);
    if (name.startsWith('Local points'))
      dimensionPoints(input, name, label.firstElementChild as HTMLElement);
    if (name.includes('(µm)')) dimension(input, name, label.firstElementChild as HTMLElement);
    input.addEventListener('input', () => {
      set(readDimension(input));
      this.changed();
    });
  }
  private render() {
    this.root.replaceChildren(element('h3', '', 'Template artwork'));
    this.root.removeAttribute('title');
    this.root.removeAttribute('aria-description');
    controlHelp(
      this.root,
      'Coordinates use the selected measurement units. Artwork may occupy margins. Apply properties commits changes.',
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
        this.root.querySelector(':scope > details:last-of-type')?.setAttribute('open', '');
      });
      this.root.append(b);
    }
    this.graphics.forEach((g, i) => {
      if (this.selectedId && g.id !== this.selectedId) return;
      const details = element('details');
      details.dataset.artworkId = g.id;
      details.open = !!this.selectedId;
      details.append(element('summary', '', `${i + 1} · ${g.kind}`));
      (g.kind === 'picture'
        ? []
        : g.kind === 'text'
          ? ['X', 'Y']
          : ['X', 'Y', 'Width', 'Height']
      ).forEach((label, n) =>
        this.field(details, `Artwork ${label} (µm)`, g.bounds[n], (v) => (g.bounds[n] = v)),
      );
      if (g.kind === 'picture' && g.picture) {
        details.append(
          new PictureFields(
            { bounds: g.bounds, picture: g.picture },
            true,
            this.changed,
            this.complete,
          ).root,
        );
      } else if (g.kind === 'text') {
        const style = { font: g.font, height: g.text_height, colour: g.colour };
        details.append(
          new FontPalette(
            [style],
            'Artwork',
            () => {
              g.font = style.font;
              g.text_height = style.height;
              g.colour = style.colour;
              this.changed();
            },
            () => {},
            { defaults: [], labels: { height: 'Artwork text height (µm)' } },
          ).root,
        );
        details.append(new TemplateRuns(g.contents, this.changed).runFields);
      } else {
        this.field(details, 'Artwork colour', g.colour, (v) => (g.colour = v), false, 'colour');
        this.field(details, 'Artwork line width (µm)', g.stroke_width, (v) => (g.stroke_width = v));
      }
      if (g.kind === 'polyline') {
        this.field(
          details,
          'Local points (µm, x,y per line)',
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
