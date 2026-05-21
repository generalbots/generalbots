const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  console.log("Navigating to embedded bot...");
  await page.goto('http://localhost:5859/embedded/index.html?bot=salesianos');

  await page.waitForTimeout(2000);

  console.log("Clicking 'ramais e cartas' suggestion...");
  // Try to find the button with text "ramais e cartas"
  try {
    await page.click('text="ramais e cartas"');
    console.log("Clicked suggestion!");
  } catch (e) {
    console.log("Suggestion not found, sending message directly...");
    await page.fill('input[type="text"]', 'ramais e cartas');
    await page.keyboard.press('Enter');
  }

  await page.waitForTimeout(8000); // wait for LLM response

  console.log("Chat Messages:");
  // Fetch messages from DOM
  const messages = await page.$$eval('.message, .chat-message', els => els.map(e => e.textContent));
  console.log(messages);

  await browser.close();
})();
