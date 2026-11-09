/** User measurement preference; never part of a document edit or undo transaction. */
import { element } from './ui';
import { lengthUnit, setLengthUnit, type LengthUnit } from './dimensions';
export function unitPicker() {
  const group = element('div', 'unit-preference');
  const label = element('label', 'settings-field');
  const select = element('select', 'property-input');
  const error = element('div', 'settings-error');
  error.setAttribute('role', 'alert');
  error.hidden = true;
  select.dataset.unitPicker = '';
  select.setAttribute('aria-label', 'Measurement units');
  for (const [key, name] of [
    ['mm', 'Millimetres (mm)'],
    ['in', 'Inches (decimal)'],
    ['um', 'Micrometres (µm)'],
  ]) {
    const option = element('option', '', name);
    option.value = key;
    select.append(option);
  }
  select.value = lengthUnit();
  select.addEventListener('change', (e) => {
    e.stopPropagation();
    try {
      setLengthUnit(select.value as LengthUnit);
      error.textContent = '';
      error.hidden = true;
    } catch (failure) {
      error.textContent = (failure as Error).message;
      error.hidden = false;
      select.value = lengthUnit();
    }
  });
  select.addEventListener('input', (e) => e.stopPropagation());
  label.append(element('span', '', 'Measurement units · user preference'), select);
  group.append(
    label,
    element(
      'p',
      'inspector-note',
      'Units save immediately for this user, independently of project changes.',
    ),
    error,
  );
  return group;
}
