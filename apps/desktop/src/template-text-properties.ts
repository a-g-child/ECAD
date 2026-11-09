/** Selection adapter: one palette for ranges, independent caption/content palettes for one cell. */
import type { TemplateTextStyles } from './types';
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
  ) {
    if (!defaults && (styles.length > 1 || allText)) {
      this.root.append(
        new FontPalette(
          styles.flatMap((s) => [s.caption, s.content]),
          'Text',
          changed,
          complete,
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
          ).root,
        );
      }
    }
  }
}
