#!/usr/bin/env node
import puppeteer from "puppeteer";

const STORYBOOK_URL = "http://localhost:6006";

async function captureStory(page, story, outputPath) {
  const url = `${STORYBOOK_URL}/iframe.html?id=${story}&viewMode=story`;
  console.log(`📸 Capturing: ${story}`);
  await page.goto(url, { waitUntil: "networkidle0" });

  // Wait a bit for any async content to load
  await page.waitForTimeout(500);

  // Find the component and take screenshot
  const element = await page.$("body");
  if (element) {
    await element.screenshot({ path: outputPath });
    console.log(`✅ Saved: ${outputPath}`);
  }
}

async function main() {
  console.log("🚀 Starting Puppeteer screenshot capture...\n");

  const browser = await puppeteer.launch({
    headless: "new",
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
  });

  const page = await browser.newPage();
  await page.setViewport({ width: 800, height: 600 });

  try {
    // Wait for Storybook to be ready
    console.log("⏳ Waiting for Storybook to be ready...");
    await page.goto(STORYBOOK_URL, { waitUntil: "networkidle0" });
    console.log("✅ Storybook is ready\n");

    // Capture different variants
    const stories = [
      { id: "prd-taskiddisplay--frontend", file: "taskid-frontend.png" },
      { id: "prd-taskiddisplay--backend", file: "taskid-backend.png" },
      { id: "prd-taskiddisplay--high-id", file: "taskid-high-id.png" },
      { id: "prd-taskiddisplay--very-high-id", file: "taskid-very-high-id.png" },
      { id: "prd-taskiddisplay--badge-variant", file: "taskid-badge.png" },
      { id: "prd-taskiddisplay--id-progression", file: "taskid-progression.png" },
    ];

    for (const story of stories) {
      await captureStory(page, story.id, `/tmp/${story.file}`);
    }

    console.log("\n✨ All screenshots captured successfully!");
    console.log("📁 Screenshots saved to /tmp/");

  } catch (error) {
    console.error("❌ Error:", error.message);
    process.exit(1);
  } finally {
    await browser.close();
  }
}

main();
