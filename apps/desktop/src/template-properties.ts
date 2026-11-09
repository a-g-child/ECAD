import { PictureFields } from './picture-controls';
/** Exact local fields form one closed property proposal; typing never edits a project. */
import { unitPicker } from './unit-picker';
import type { TemplateVariant, TemplateCell, TemplateTable, TemplateRun } from './types';
import { element } from './ui';
import { TemplateTextProperties } from './template-text-properties';
import { dimension, dimensionSummary, readDimension, writeDimension } from './dimensions';
import { textEditor } from './text';
import { paperPresets } from './paper-presets';
import { TemplateRuns } from './template-runs';
import { TemplateArtwork } from './template-artwork';
export class TemplateProperties {
  readonly root = element('form', 'template-properties inspector');
  private blanket = false;
  private cellPicture: PictureFields | null = null;
  private textDefaults: TemplateTextProperties | null = null;
  private cellStyles: { id: string; styles: import('./types').TemplateTextStyles }[] = [];
  private paperFields = element('div');
  private artworkId: string | undefined;
  private catalog = false;
  private advancedPaper = false;
  private name = textEditor('property-input property-text', 2);
  private paper: HTMLInputElement[] = [];
  private content: HTMLInputElement[] = [];
  private generalMargin = element('input', 'property-input');
  private fitMargins = element('input');
  private caption = textEditor('property-input property-text', 2);
  private runs: TemplateRun[] = [];
  private artwork: TemplateArtwork | null = null;
  private selected: { table: TemplateTable; cell: TemplateCell } | null = null;
  private model: TemplateVariant | null = null;
  constructor(
    private changed: () => void,
    private apply: () => void,
  ) {
    this.root.addEventListener('submit', (e) => e.preventDefault());
    this.root.addEventListener('keydown', (e) => {
      if (e.isComposing) return;
      if (this.catalog && e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        apply();
      }
    });
  }
  private label(name: string, input: HTMLElement, parent: HTMLElement = this.paperFields) {
    const label = element('label', 'settings-field');
    label.append(element('span', '', name), input);
    parent.append(label);
    input.setAttribute('aria-label', name);
    input.addEventListener('input', this.changed);
    input.addEventListener('change', this.changed);
    if (name.includes('(µm)') && input instanceof HTMLInputElement)
      dimension(input, name, label.firstElementChild as HTMLElement);
    return label;
  }
  private numbers(names: string[], values: string[]) {
    return names.map((label, i) => {
      const input = element('input', 'property-input');
      input.inputMode = 'numeric';
      input.value = values[i];
      this.label(label, input);
      return input;
    });
  }
  render(
    model: TemplateVariant,
    selection: { table: TemplateTable; cell: TemplateCell } | null,
    artworkId?: string,
    catalog = false,
    cells: TemplateCell[] = [],
    wholeTable = false,
  ) {
    this.model = model;
    this.blanket = wholeTable || cells.length > 1;
    this.selected = selection;
    this.artworkId = artworkId;
    this.catalog = catalog;
    this.paperFields = element('div');
    this.paperFields.hidden = !!selection || !!artworkId || catalog;
    this.root.replaceChildren(this.paperFields);
    this.root.scrollTop = 0;
    this.name = textEditor('property-input property-text', 2);
    this.name.value = model.name;
    this.label('Template name', this.name);
    this.paper = this.numbers(['Paper width (µm)', 'Paper height (µm)'], model.paper.slice(0, 2));
    this.generalMargin = element('input', 'property-input');
    this.generalMargin.inputMode = 'numeric';
    this.generalMargin.title = 'Set all four margins together. Individual margins remain editable.';
    this.label('General margin (µm)', this.generalMargin);
    this.paper.push(
      ...this.numbers(
        ['Left margin (µm)', 'Right margin (µm)', 'Top margin (µm)', 'Bottom margin (µm)'],
        model.paper.slice(2),
      ),
    );
    const syncGeneral = () => {
      const values = this.paper.slice(2).map((input) => input.value);
      const uniform = values.every((value) => value === values[0]);
      writeDimension(this.generalMargin, uniform ? readDimension(this.paper[2]) : '');
      this.generalMargin.placeholder = uniform ? '' : 'Mixed';
    };
    syncGeneral();
    for (const side of this.paper.slice(2)) {
      side.addEventListener('input', syncGeneral);
      side.addEventListener('change', syncGeneral);
    }
    this.fitMargins = element('input');
    this.fitMargins.type = 'checkbox';
    this.fitMargins.title =
      'Move tables only as needed and intersect the content region with the margins. Cell sizes stay unchanged.';
    this.label('Fit layout inside margins', this.fitMargins);
    this.generalMargin.addEventListener('input', () => {
      for (const side of this.paper.slice(2))
        writeDimension(side, readDimension(this.generalMargin));
      this.fitMargins.checked = true;
    });
    this.content = this.numbers(
      ['Content X (µm)', 'Content Y (µm)', 'Content width (µm)', 'Content height (µm)'],
      model.content,
    );
    this.paperFields.append(
      element(
        'p',
        'inspector-note',
        'Paper sizes are exact; contents are not scaled. Margin profiles and border styles remain separate from paper size.',
      ),
    );
    paperPresets(
      this.paperFields,
      this.paper,
      this.content,
      this.fitMargins,
      this.changed,
      this.advancedPaper,
      (open) => {
        this.advancedPaper = open;
      },
    );
    this.textDefaults = new TemplateTextProperties(
      [structuredClone(model.text_defaults)],
      this.changed,
      true,
      this.apply,
    );
    this.paperFields.append(this.textDefaults.root, unitPicker());
    this.cellStyles = cells
      .filter((c) => !c.picture)
      .map((c) => ({ id: c.id, styles: structuredClone(c.text_styles) }));
    this.artwork = new TemplateArtwork(model, this.changed, artworkId, () => {
      if (!this.catalog) this.apply();
    });
    if (artworkId || catalog) this.root.append(this.artwork.root);
    this.cellPicture = null;
    this.runs = [];
    if (selection) {
      this.root.append(
        element('h3', '', this.blanket ? `${cells.length} cells selected` : 'Cell properties'),
      );
      if (!this.blanket) {
        this.caption = textEditor('property-input property-text', 2);
        this.caption.value = selection.cell.caption;
        if (!selection.cell.picture) this.label('Cell caption', this.caption, this.root);
        this.root.append(dimensionSummary(selection.cell.bounds, 'Cell bounds · '));
        this.runs = selection.cell.contents.map((r) => ({ ...r }));
        if (!selection.cell.picture) {
          this.root.append(new TemplateRuns(this.runs, this.changed).runFields);
          this.root.append(
            element(
              'p',
              'settings-note',
              'Choosing a picture replaces the cell caption and content.',
            ),
          );
        } else if (selection.cell.caption || selection.cell.contents.length) {
          this.root.append(
            element(
              'p',
              'settings-note',
              'This older cell contains text and a picture. Choose which content to keep.',
            ),
          );
          const keep = element('button', 'plain', 'Keep picture only');
          keep.type = 'button';
          keep.addEventListener('click', () => {
            this.caption.value = '';
            this.runs.splice(0);
            this.changed();
            this.apply();
          });
          this.root.append(keep);
        }
        this.cellPicture = new PictureFields(
          {
            bounds: selection.cell.bounds,
            picture: structuredClone(
              selection.cell.picture ?? { data: '', mime: 'image/png', fit: 'contain' },
            ),
          },
          false,
          this.changed,
          () => {
            this.caption.value = '';
            this.runs.splice(0);
            this.apply();
          },
        );
        this.root.append(this.cellPicture.root);
        if (selection.cell.picture) {
          const remove = element('button', 'plain', 'Use text instead');
          remove.type = 'button';
          remove.addEventListener('click', () => {
            this.cellPicture!.value.picture.data = '';
            this.changed();
            this.apply();
          });
          this.cellPicture.root.append(remove);
        }
      }
      if (!this.cellStyles.length && !selection.cell.picture && !this.blanket)
        this.cellStyles = [
          { id: selection.cell.id, styles: structuredClone(selection.cell.text_styles) },
        ];
      if (this.cellStyles.length)
        this.root.append(
          new TemplateTextProperties(
            this.cellStyles.map((c) => c.styles),
            this.changed,
            false,
            this.apply,
            this.blanket,
            this.model?.text_defaults.layout,
          ).root,
        );
    }
  }

  proposal() {
    if (!this.model) throw new Error('Template is closed');
    return {
      variant: this.model.id,
      name: this.name.value,
      paper: this.paper.map(readDimension),
      text_defaults: this.textDefaults!.styles[0],
      cell_styles: this.cellStyles,
      fit_margins: this.fitMargins.checked,
      content: this.content.map(readDimension),
      graphics: this.artwork!.graphics,
      cell:
        this.selected && !this.blanket
          ? {
              table: this.selected.table.id,
              id: this.selected.cell.id,
              picture: this.cellPicture?.value.picture.data ? this.cellPicture.value.picture : null,
              caption: this.caption.value,
              contents: this.runs,
            }
          : null,
    };
  }
  focusArtwork(id?: string) {
    if (!id) {
      const section = this.root.querySelector<HTMLElement>('.template-artwork');
      section?.scrollIntoView({ block: 'nearest' });
      section?.querySelector('button')?.focus();
      return;
    }
    const detail = this.root.querySelector<HTMLDetailsElement>(`[data-artwork-id="${id}"]`);
    if (detail) {
      detail.open = true;
      detail.scrollIntoView({ block: 'nearest' });
      detail.querySelector<HTMLInputElement>('input,textarea')?.focus();
    }
  }
  focus() {
    if (this.artworkId || this.catalog) {
      this.focusArtwork(this.artworkId);
      return;
    }
    if (this.blanket)
      this.root
        .querySelector<HTMLInputElement>('.font-palette textarea,.font-palette input')
        ?.focus();
    else (this.selected ? this.caption : this.name).focus();
  }
}
