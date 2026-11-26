'use strict';

async function readFileBounded(fsPromises, filePath, maxBytes, label = 'Soubor') {
  const limit = Math.max(1, Math.min(50_000_000, Math.trunc(Number(maxBytes) || 1_000_000)));
  const handle = await fsPromises.open(filePath, 'r');
  const chunks = [];
  const buffer = Buffer.alloc(64 * 1024);
  let total = 0;
  try {
    while (true) {
      const remainingWithProbe = limit + 1 - total;
      if (remainingWithProbe <= 0) throw new Error(`${label} překročil limit ${Math.round(limit / 1024)} kB.`);
      const length = Math.min(buffer.byteLength, remainingWithProbe);
      const { bytesRead } = await handle.read(buffer, 0, length, null);
      if (!bytesRead) break;
      total += bytesRead;
      if (total > limit) throw new Error(`${label} překročil limit ${Math.round(limit / 1024)} kB.`);
      chunks.push(Buffer.from(buffer.subarray(0, bytesRead)));
    }
    return Buffer.concat(chunks, total);
  } finally {
    await handle.close();
  }
}

module.exports = Object.freeze({ readFileBounded });
