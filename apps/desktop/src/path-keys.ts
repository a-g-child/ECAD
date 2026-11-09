/** Shared path editing keys; text/IME ownership remains with each editor's input boundary. */
export function pathCommand(event: KeyboardEvent): 'finish' | 'back' | 'cancel' | null {
  if (event.isComposing) return null;
  const command = { Enter: 'finish', Backspace: 'back', Escape: 'cancel' } as const;
  if (!Object.hasOwn(command, event.key)) return null;
  event.preventDefault();
  // A held key must not consume multiple vertices or create repeated finish requests.
  return event.repeat ? null : command[event.key as keyof typeof command];
}
