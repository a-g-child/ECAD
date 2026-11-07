/** Shared controls: text is always assigned as textContent, including imported project labels. */
export function element<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className = '',
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}
const icons: Record<string, string> = {
  open: 'M2 6h6l2 2h12v12H2z M2 6V3h7l2 3',
  save: 'M4 3h14l3 3v15H3V3z M7 3v6h10V3 M7 21v-8h10v8',
  undo: 'M8 5 3 10l5 5 M3 10h10a7 7 0 0 1 7 7',
  redo: 'm16 5 5 5-5 5 M21 10H11a7 7 0 0 0-7 7',
  fit: 'M3 8V3h5 M16 3h5v5 M21 16v5h-5 M8 21H3v-5 M7 7h10v10H7z',
  properties: 'M4 5h16 M4 12h16 M4 19h16 M8 3v4 M16 10v4 M10 17v4',
  path: 'M3 18 9 5l6 11 6-12 M2 17h2v2H2z M8 4h2v2H8z M14 15h2v2h-2z M20 3h2v2h-2z',
  text: 'M4 4h16 M12 4v17 M8 21h8 M4 4v4 M20 4v4',
  rotate: 'M5 7a9 9 0 1 1-1 9 M5 2v5h5',
  mirror: 'M12 2v20 M3 6v12l6-6z M21 6v12l-6-6z',
  move: 'M12 2v20 M2 12h20 M8 6l4-4 4 4 M8 18l4 4 4-4 M6 8l-4 4 4 4 M18 8l4 4-4 4',
  select: 'm6 3 13 10-7 1-3 7z',
  report: 'M5 3h14v18H5z M8 8h8 M8 12h8 M8 16h5',
  search: 'M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16 M16 16l6 6',
};
export function icon(name: string): SVGSVGElement {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  svg.classList.add('icon');
  const path = document.createElementNS(svg.namespaceURI, 'path');
  path.setAttribute('d', icons[name] ?? icons.properties);
  svg.append(path);
  return svg;
}
export function tool(label: string, name: string, callback: () => void) {
  const button = element('button', 'tool');
  button.type = 'button';
  button.title = label;
  button.setAttribute('aria-label', label);
  button.append(icon(name));
  button.addEventListener('click', callback);
  return button;
}
export function separator() {
  return element('span', 'separator');
}
