/** Shared image import, preview and exact placement fields; owners supply transaction boundaries. */
import { element } from './ui';
import { dimension, readDimension, writeDimension } from './dimensions';
import { send, errorText } from './transport';
export interface Picture {
  png: string;
  fit: 'contain' | 'cover' | 'stretch';
}
export interface PicturePlacement {
  bounds: string[];
  picture: Picture;
}
export class PictureFields {
  readonly root = element('section', 'picture-properties');
  private preview = element('img', 'picture-preview');
  private error = element('p', 'settings-error');
  private fields: HTMLInputElement[] = [];
  private importing = false;
  get pending() {
    return this.importing;
  }
  constructor(
    readonly value: PicturePlacement,
    geometry: boolean,
    private changed: () => void,
    complete: () => void,
  ) {
    this.preview.alt = 'Picture preview';
    this.error.setAttribute('role', 'alert');
    const browse = element('button', 'plain', 'Browse PNG…');
    browse.type = 'button';
    const file = element('input');
    file.type = 'file';
    file.accept = 'image/png,.png';
    file.hidden = true;
    file.setAttribute('aria-label', 'Picture file');
    browse.addEventListener('click', () => file.click());
    file.addEventListener('change', async (e) => {
      e.stopPropagation();
      const chosen = file.files?.[0];
      if (!chosen) return;
      this.importing = true;
      browse.disabled = true;
      this.error.textContent = '';
      try {
        if (chosen.size > 4 * 1024 * 1024) throw new Error('Choose a PNG up to 4 MiB.');
        const png = await new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => resolve(String(reader.result).split(',')[1]);
          reader.onerror = () => reject(new Error('Could not read this picture.'));
          reader.readAsDataURL(chosen);
        });
        const result = (await send({ type: 'import_picture', png })).imported_picture;
        if (!this.root.isConnected || !result) return;
        const fresh = !value.picture.png;
        value.picture.png = result.picture.png;
        if (fresh && geometry) {
          value.bounds[3] = String(
            Math.max(
              1,
              Math.round(
                (Number(value.bounds[2]) * Number(result.pixels[1])) / Number(result.pixels[0]),
              ),
            ),
          );
          writeDimension(this.fields[3], value.bounds[3]);
        }
        this.show();
        this.changed();
        this.importing = false;
        complete();
      } catch (e) {
        this.error.textContent = errorText(e);
      } finally {
        this.importing = false;
        browse.disabled = false;
        file.value = '';
      }
    });
    this.root.append(element('h4', '', 'Picture'), this.preview, browse, file);
    if (geometry)
      ['X', 'Y', 'Width', 'Height'].forEach((name, i) => {
        const input = element('input', 'property-input');
        input.value = value.bounds[i];
        const label = element('label', 'settings-field'),
          text = element('span', '', `Picture ${name} (µm)`);
        label.append(text, input);
        dimension(input, `Picture ${name} (µm)`, text);
        input.addEventListener('input', () => {
          value.bounds[i] = readDimension(input);
          this.changed();
        });
        this.fields.push(input);
        this.root.append(label);
      });
    const fit = element('select', 'property-input');
    fit.setAttribute('aria-label', 'Picture fit');
    for (const [key, label] of [
      ['contain', 'Fit whole image'],
      ['cover', 'Fill and crop'],
      ['stretch', 'Stretch'],
    ]) {
      const option = element('option', '', label);
      option.value = key;
      fit.append(option);
    }
    fit.value = value.picture.fit;
    fit.addEventListener('change', () => {
      value.picture.fit = fit.value as Picture['fit'];
      this.show();
      this.changed();
    });
    const fitLabel = element('label', 'settings-field');
    fitLabel.append(element('span', '', 'Picture fit'), fit);
    this.root.append(
      fitLabel,
      element(
        'p',
        'settings-note',
        'A local copy is saved with the project or template. PNG · up to 4 MiB and 4 megapixels.',
      ),
      this.error,
    );
    this.show();
  }
  private show() {
    this.preview.hidden = !this.value.picture.png;
    if (this.value.picture.png)
      this.preview.src = `data:image/png;base64,${this.value.picture.png}`;
    this.preview.style.objectFit =
      this.value.picture.fit === 'stretch' ? 'fill' : this.value.picture.fit;
  }
}
