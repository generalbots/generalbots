import { chromium } from 'playwright';

const browser = await chromium.launch({ 
  headless: true,
  args: ['--no-sandbox']
});

const context = await browser.newContext({
  viewport: { width: 1280, height: 720 }
});
const page = await context.newPage();

// Collect console messages
page.on('console', msg => console.log(`CONSOLE: ${msg.type()}: ${msg.text()}`));
page.on('pageerror', err => console.log(`PAGE_ERR: ${err.message}`));

console.log('Navigating to cristo bot...');
await page.goto('http://localhost:3000/cristo', { waitUntil: 'networkidle', timeout: 15000 });
await page.waitForTimeout(3000);

// Screenshot 1: Initial page load
await page.screenshot({ path: '/tmp/cristo-01-initial.png' });
console.log('Screenshot 1: initial page');

// Find chat input and type message
const chatInput = await page.$('textarea, input[type="text"], .chat-input, [contenteditable]');
if (chatInput) {
  const tagName = await chatInput.evaluate(el => el.tagName);
  console.log(`Chat input found: ${tagName}`);
  
  // Type a message
  await chatInput.click();
  await chatInput.fill('Quero agendar um batizado');
  await chatInput.press('Enter');
  console.log('Message sent: Quero agendar um batizado');
  
  // Wait for response
  await page.waitForTimeout(8000);
  await page.screenshot({ path: '/tmp/cristo-02-after-batizado.png' });
  console.log('Screenshot 2: after batizado request');
  
  // Check for more messages loaded
  const messages = await page.$$('.message, .chat-message, [class*="msg"]');
  console.log(`Messages found: ${messages.length}`);
  
  // Try to send child name
  await chatInput.click();
  await chatInput.fill('João Pedro Silva');
  await chatInput.press('Enter');
  console.log('Sent child name');
  await page.waitForTimeout(8000);
  await page.screenshot({ path: '/tmp/cristo-03-after-name.png' });
  console.log('Screenshot 3: after providing name');
  
  // Try clicking suggestion buttons
  const buttons = await page.$$('button, .suggestion, [class*="suggest"]');
  console.log(`Buttons found: ${buttons.length}`);
  for (const btn of buttons) {
    const text = await btn.textContent();
    console.log(`  Button: ${text.substring(0, 50)}`);
  }
} else {
  console.log('No chat input found - taking page snapshot');
  const html = await page.content();
  console.log(html.substring(0, 2000));
}

// Final screenshot
await page.screenshot({ path: '/tmp/cristo-04-final.png' });
console.log('Screenshot 4: final state');

console.log('\nScreenshots saved to /tmp/cristo-*.png');
await browser.close();
