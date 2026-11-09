/** Non-layout control guidance shared by inspectors and dialogs. */
export function controlHelp(control: HTMLElement, text: string) {
  const description = control.getAttribute('aria-description') ?? control.title;
  control.title = [control.title, text].filter(Boolean).join('\n');
  control.setAttribute('aria-description', [description, text].filter(Boolean).join('\n'));
}
