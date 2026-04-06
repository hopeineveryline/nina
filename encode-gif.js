// Encodes PNG frames captured by capture-demo.js into an animated GIF.
// Usage: node encode-gif.js [fps] [loop]
// Defaults: 15fps, loop forever (0)

const { createCanvas, loadImage } = require('canvas');
const GIFEncoder = require('gifencoder');
const fs = require('fs');
const path = require('path');

const fps = parseInt(process.argv[2] ?? '15', 10);
const loop = parseInt(process.argv[3] ?? '0', 10); // 0 = infinite
const framesDir = path.resolve(__dirname, '_demo_frames');
const outPath = path.resolve(__dirname, 'demo.gif');

const WIDTH = 360;
const HEIGHT = 230;

async function main() {
  const files = fs.readdirSync(framesDir)
    .filter(f => f.endsWith('.png'))
    .sort()
    .filter((_, i) => i % 5 === 0) // every 5th frame
    .slice(0, 40); // ~2s at 20fps → 40 frames

  if (files.length === 0) {
    console.error('No PNG frames found. Run capture-demo.js first.');
    process.exit(1);
  }

  console.log(`Encoding ${files.length} frames at ${fps}fps → ${outPath}`);

  const encoder = new GIFEncoder(WIDTH, HEIGHT);
  encoder.setDelay(Math.round(1000 / fps));
  encoder.setRepeat(loop);
  encoder.setQuality(20); // 1 = best/slow, 20 = fastest/worst

  const stream = encoder.createReadStream();
  const out = fs.createWriteStream(outPath);
  stream.pipe(out);

  encoder.start();

  for (let i = 0; i < files.length; i++) {
    const imgPath = path.join(framesDir, files[i]);
    const img = await loadImage(imgPath);
    const canvas = createCanvas(WIDTH, HEIGHT);
    const ctx = canvas.getContext('2d');
    ctx.drawImage(img, 0, 0, WIDTH, HEIGHT);
    encoder.addFrame(ctx);

    if (i % 20 === 0) {
      process.stdout.write(`  frame ${i}/${files.length}\r`);
    }
  }

  encoder.finish();
  out.on('finish', () => {
    const size = (fs.statSync(outPath).size / 1024).toFixed(1);
    console.log(`\nDone. demo.gif (${size}KB)`);
  });
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
