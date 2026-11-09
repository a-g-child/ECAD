/** Schematic adapter for shared picture controls and contextual completion. */
import { PictureFields, type PicturePlacement } from './picture-controls';
import { PropertyCompletion } from './property-completion';
import { element } from './ui';
export function pictureProperties(
  parent: HTMLElement,
  value: PicturePlacement,
  commit: (value: PicturePlacement | null) => Promise<boolean>,
  revert: () => Promise<void>,
  busy: () => boolean,
) {
  let pending = false;
  const finish = async () => {
    if (!pending || busy() || fields.pending) return !pending;
    const okay = await commit(structuredClone(fields.value));
    if (okay) pending = false;
    return okay;
  };
  const fields = new PictureFields(
    structuredClone(value),
    true,
    () => {
      pending = true;
    },
    () => void finish(),
  );
  new PropertyCompletion(
    fields.root,
    () => true,
    () => pending,
    () => busy() || fields.pending,
    finish,
    revert,
  );
  const remove = element('button', 'plain', 'Remove picture');
  remove.type = 'button';
  remove.addEventListener('click', () => void commit(null));
  fields.root.append(remove);
  parent.prepend(fields.root);
}
