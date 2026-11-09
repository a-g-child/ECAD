/** Selection adapter: one palette for ranges, independent caption/content palettes for one cell. */
import { CellTextLayoutProperties } from './cell-text-layout-properties';
import type { TemplateTextStyles, TemplateCellTextLayout } from './types';
import { element } from './ui';
import { FontPalette } from './font-palette';
export class TemplateTextProperties {
  readonly root = element('section', 'template-text-properties');
  constructor(
    readonly styles: TemplateTextStyles[],
    changed: () => void,
    defaults = false,
    complete: () => void = () => {},
    allText = false,
    layoutDefaults?: TemplateCellTextLayout,
  ) {
    const hasLayout = styles.some((s) =>
      [s.layout, layoutDefaults].some((l) => Object.values(l ?? {}).some((v) => v != null)),
    );
    const inline = styles.every(
      (s) => (s.layout?.caption_position ?? layoutDefaults?.caption_position) === 'inline',
    );
    const options: import('./font-properties').FontPaletteOptions = {
      exclude: hasLayout ? (inline ? ['horizontal', 'vertical'] : ['vertical']) : [],
    };
    this.root.append(new CellTextLayoutProperties(styles, changed, defaults, complete).root);
    if (!defaults && (styles.length > 1 || allText)) {
      this.root.append(
        new FontPalette(
          styles.flatMap((s) => [s.caption, s.content]),
          'Text',
          changed,
          complete,
          options,
        ).root,
      );
      this.root.append(
        element(
          'p',
          'inspector-note',
          'Formatting applies to captions and contents in every selected cell. Other properties keep their existing values.',
        ),
      );
    } else {
      for (const channel of ['caption', 'content'] as const) {
        const title = `${channel === 'caption' ? 'Caption' : 'Content'}${defaults ? ' default' : ''}`;
        this.root.append(
          new FontPalette(
            styles.map((s) => s[channel]),
            title,
            changed,
            complete,
            options,
          ).root,
        );
      }
    }
  }
}
