const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

(async () => {
  const framesDir = path.resolve(__dirname, '_demo_frames');
  fs.rmSync(framesDir, { recursive: true, force: true });
  fs.mkdirSync(framesDir);

  const browser = await chromium.launch({ args: ['--font-render-hinting=none'] });
  const page = await browser.newPage();

  await page.setViewportSize({ width: 720, height: 460 });

  // load local file
  const htmlPath = path.resolve(__dirname, 'demo.html');
  await page.goto(`file://${htmlPath}`);

  // let the page initialize
  await page.waitForTimeout(200);

  const fps = 20;
  const durationMs = 24000; // 24 seconds = one full loop of all 3 scenes
  const frameCount = Math.ceil((durationMs / 1000) * fps);
  const interval = 1000 / fps;

  console.log(`capturing ${frameCount} frames at ${fps}fps (${durationMs / 1000}s)...`);

  for (let i = 0; i < frameCount; i++) {
    const framePath = path.join(framesDir, `f${String(i).padStart(5, '0')}.png`);
    await page.screenshot({ path: framePath });
    if (i % 20 === 0) process.stdout.write(`  frame ${i}/${frameCount}\r`);
    await page.waitForTimeout(interval);
  }

  console.log(`\ndone. ${frameCount} frames in ${framesDir}`);
  await browser.close();
})();
