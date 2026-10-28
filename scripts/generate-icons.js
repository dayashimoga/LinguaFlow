import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

function createCrcTable() {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      if (c & 1) c = 0xedb88320 ^ (c >>> 1);
      else c = c >>> 1;
    }
    table[n] = c;
  }
  return table;
}

const crcTable = createCrcTable();
function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = crcTable[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makeChunk(type, data) {
  const typeBuf = Buffer.from(type, 'ascii');
  const lenBuf = Buffer.alloc(4);
  lenBuf.writeUInt32BE(data.length, 0);

  const crcBuf = Buffer.alloc(4);
  const toCrc = Buffer.concat([typeBuf, data]);
  crcBuf.writeUInt32BE(crc32(toCrc), 0);

  return Buffer.concat([lenBuf, typeBuf, data, crcBuf]);
}

function generatePng(width, height) {
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  // IHDR
  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData[8] = 8; // bit depth
  ihdrData[9] = 6; // color type: RGBA
  ihdrData[10] = 0; // compression
  ihdrData[11] = 0; // filter
  ihdrData[12] = 0; // interlace
  const ihdrChunk = makeChunk('IHDR', ihdrData);

  // Scanlines with a gradient blue-indigo color
  const scanlines = [];
  for (let y = 0; y < height; y++) {
    scanlines.push(0); // filter: none
    for (let x = 0; x < width; x++) {
      const r = Math.floor(14 + (85 * x) / width);
      const g = Math.floor(165 - (60 * y) / height);
      const b = Math.floor(233 + (22 * (x + y)) / (2 * width));
      scanlines.push(r, g, b, 255);
    }
  }

  const rawData = Buffer.from(scanlines);
  const compressed = zlib.deflateSync(rawData);
  const idatChunk = makeChunk('IDAT', compressed);
  const iendChunk = makeChunk('IEND', Buffer.alloc(0));

  return Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
}

function generateIco(pngBuffer, width, height) {
  // ICO header: 6 bytes
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // 1 = ICO
  header.writeUInt16LE(1, 4); // 1 image

  // Directory entry: 16 bytes
  const entry = Buffer.alloc(16);
  entry[0] = width >= 256 ? 0 : width;
  entry[1] = height >= 256 ? 0 : height;
  entry[2] = 0; // no palette
  entry[3] = 0; // reserved
  entry.writeUInt16LE(1, 4); // color planes
  entry.writeUInt16LE(32, 6); // bpp
  entry.writeUInt32LE(pngBuffer.length, 8); // image size
  entry.writeUInt32LE(22, 12); // image offset (6 header + 16 entry = 22)

  return Buffer.concat([header, entry, pngBuffer]);
}

const iconsDir = path.resolve('src-tauri/icons');
fs.mkdirSync(iconsDir, { recursive: true });

const png32 = generatePng(32, 32);
const png128 = generatePng(128, 128);
const png256 = generatePng(256, 256);
const ico = generateIco(png32, 32, 32);

fs.writeFileSync(path.join(iconsDir, '32x32.png'), png32);
fs.writeFileSync(path.join(iconsDir, '128x128.png'), png128);
fs.writeFileSync(path.join(iconsDir, '128x128@2x.png'), png256);
fs.writeFileSync(path.join(iconsDir, 'icon.png'), png256);
fs.writeFileSync(path.join(iconsDir, 'icon.ico'), ico);
fs.writeFileSync(path.join(iconsDir, 'icon.icns'), png256); // Dummy placeholder for macOS icns bundle

console.log('Icons generated successfully in src-tauri/icons');
