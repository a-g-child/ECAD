/** Native colour selection stages pointer input; accepted changes use the owner's existing edit path. */
import { element } from './ui';
export function colourPicker(exact: HTMLInputElement | HTMLTextAreaElement, chosen?: () => void) {
  const root = element('div', 'colour-picker');
  const picker = element('input', 'property-input');
  picker.type = 'color';
  picker.setAttribute('aria-label', `${exact.getAttribute('aria-label')} picker`);
  const hint = element('span', 'colour-picker-hint');
  const sync = () => {
    const valid = /^#[0-9a-f]{6}$/i.test(exact.value);
    picker.value = valid ? exact.value.toLowerCase() : '#000000';
    root.classList.toggle('colour-unset', !valid);
    hint.textContent =
      exact.placeholder === 'Mixed' && !exact.value ? 'Mixed colours' : 'Choose colour';
    hint.hidden = valid;
    picker.title = valid
      ? 'Choose colour'
      : `${hint.textContent} — no value changes until you choose`;
  };
  exact.addEventListener('input', sync);
  picker.addEventListener('change', () => {
    exact.value = picker.value;
    exact.dispatchEvent(new Event('input', { bubbles: true }));
    chosen?.();
  });
  // Native input events can repeat while dragging. Only change completes an edit.
  sync();
  root.append(picker, hint);
  return root;
}
