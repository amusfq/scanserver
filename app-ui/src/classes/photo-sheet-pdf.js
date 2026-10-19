const encoder = new TextEncoder();

export default function photoSheetPdf(images, widthMm, heightMm) {
  const objects = new Array(3 + images.length * 3);
  const kids = images.map((_, index) => `${3 + index * 3} 0 R`).join(' ');
  objects[1] = '<< /Type /Catalog /Pages 2 0 R >>';
  objects[2] = `<< /Type /Pages /Kids [${kids}] /Count ${images.length} >>`;

  for (let index = 0; index < images.length; index += 1) {
    const image = images[index];
    const imageBytes = image.jpeg;
    const pageId = 3 + index * 3;
    const imageId = pageId + 1;
    const contentId = pageId + 2;
    const widthPt = (widthMm * 72 / 25.4).toFixed(3);
    const heightPt = (heightMm * 72 / 25.4).toFixed(3);
    const content = encoder.encode(`q ${widthPt} 0 0 ${heightPt} 0 0 cm /Photo Do Q`);
    objects[pageId] = encoder.encode(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${widthPt} ${heightPt}] /Resources << /XObject << /Photo ${imageId} 0 R >> >> /Contents ${contentId} 0 R >>`);
    objects[imageId] = [
      encoder.encode(`<< /Type /XObject /Subtype /Image /Width ${image.width} /Height ${image.height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length ${imageBytes.length} >>\nstream\n`),
      imageBytes,
      encoder.encode('\nendstream')
    ];
    objects[contentId] = [
      encoder.encode(`<< /Length ${content.length} >>\nstream\n`),
      content,
      encoder.encode('\nendstream')
    ];
  }

  const chunks = [encoder.encode('%PDF-1.4\n')];
  const offsets = [0];
  let length = chunks[0].length;
  for (let index = 1; index < objects.length; index += 1) {
    offsets[index] = length;
    const body = objects[index];
    const chunksForObject = [encoder.encode(`${index} 0 obj\n`), ...(Array.isArray(body) ? body : [body]), encoder.encode('\nendobj\n')];
    chunks.push(...chunksForObject);
    length += chunksForObject.reduce((sum, chunk) => sum + chunk.length, 0);
  }

  const xrefOffset = length;
  const xref = `xref\n0 ${objects.length}\n0000000000 65535 f \n${offsets.slice(1).map(offset => `${String(offset).padStart(10, '0')} 00000 n \n`).join('')}trailer\n<< /Size ${objects.length} /Root 1 0 R >>\nstartxref\n${xrefOffset}\n%%EOF`;
  chunks.push(encoder.encode(xref));
  return new Blob(chunks, { type: 'application/pdf' });
}
