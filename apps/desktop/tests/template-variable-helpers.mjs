/** Exercise the same picker used by template properties and canvas text placement. */
export async function insertVariable(scope, key) {
  await scope.getByRole('button', { name: 'Insert variable', exact: true }).click();
  const picker = scope.getByRole('dialog', { name: 'Insert variable', exact: true });
  await picker.getByRole('button', { name: key, exact: true }).click();
  await picker.waitFor({ state: 'hidden' });
}
