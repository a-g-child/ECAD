/** One serialized request stream; both desktop IPC and the browser harness call the real Rust bridge. */
import { invoke, isTauri } from '@tauri-apps/api/core';
import { validateReply, type Reply } from './types';
export const native = isTauri();
let tail: Promise<unknown> = Promise.resolve();
let input = 0n;
export function syncInput(next: string) {
  const observed = BigInt(next) - 1n;
  if (observed > input) input = observed;
}
export function inputId() {
  input += 1n;
  return input.toString();
}
export function send(request: object): Promise<Reply> {
  const operation = tail
    .catch(() => undefined)
    .then(async () => {
      if (native) return validateReply(await invoke('bridge', { request }));
      const response = await fetch('/bridge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      });
      const body = await response.json();
      if (body.error) throw body.error;
      return validateReply(body.ok);
    });
  tail = operation;
  return operation;
}
export async function chooseFile(save: boolean): Promise<Reply | null> {
  if (!native)
    throw new Error(
      'File choosers are available in the desktop app. The browser harness uses a temporary project.',
    );
  const reply = await invoke<unknown>('choose_file', { save });
  return reply === null ? null : validateReply(reply);
}
export function errorText(error: unknown): string {
  if (error && typeof error === 'object' && 'message' in error) return String(error.message);
  return String(error);
}
