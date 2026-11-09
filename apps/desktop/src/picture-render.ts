/** One raster mapping for schematic, template and cell-bound pictures; bytes stay immutable. */
import type { Picture } from './picture-controls';
let serial = 0;
export function renderPicture(
  parent: SVGElement,
  picture: Picture,
  bounds: { x: number; y: number; width: number; height: number },
) {
  const ns = 'http://www.w3.org/2000/svg';
  const image = document.createElementNS(ns, 'image');
  for (const [key, value] of Object.entries(bounds)) image.setAttribute(key, String(value));
  image.setAttribute('href', `data:${picture.mime};base64,${picture.data}`);
  image.setAttribute(
    'preserveAspectRatio',
    picture.fit === 'stretch' ? 'none' : `xMidYMid ${picture.fit === 'cover' ? 'slice' : 'meet'}`,
  );
  image.classList.add('picture-image');
  const clip = document.createElementNS(ns, 'clipPath'),
    rect = document.createElementNS(ns, 'rect');
  const id = `picture-clip-${++serial}`;
  clip.id = id;
  for (const [key, value] of Object.entries(bounds)) rect.setAttribute(key, String(value));
  clip.append(rect);
  image.setAttribute('clip-path', `url(#${id})`);
  parent.append(clip, image);
  return image;
}
