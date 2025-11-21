# Quick Start Guide - General Bots Desktop

Get up and running with General Bots Desktop in 5 minutes!

## 🚀 Installation

### Option 1: Using Python (Recommended)

```bash
cd botserver/web/desktop
python3 -m http.server 8000
```

Open http://localhost:8000 in your browser.

### Option 2: Using Node.js

```bash
cd botserver/web/desktop
npx http-server -p 8000
```

### Option 3: Using PHP

```bash
cd botserver/web/desktop
php -S localhost:8000
```

## 📂 Project Structure at a Glance

```
desktop/
├── index.html              # Main entry point
├── css/app.css             # Core styles + theme system
├── js/
│   ├── theme-manager.js    # Theme switching
│   └── layout.js           # Section loading
├── public/themes/          # 19+ theme files
├── chat/                   # Chat module
├── drive/                  # Drive module
├── tasks/                  # Tasks module
└── mail/                   # Mail module
```

## 🎨 Try Different Themes

1. Launch the application
2. Click the theme dropdown in the header
3. Select any theme (try "Cyberpunk" or "Retrowave"!)
4. Theme is saved automatically

## ⌨️ Essential Keyboard Shortcuts

- **Alt + 1** → Chat
- **Alt + 2** → Drive
- **Alt + 3** → Tasks
- **Alt + 4** → Mail
- **Esc** → Close menus

## 🛠️ Create Your First Theme

### Step 1: Create Theme File

Create `public/themes/myawesome.css`:

```css
:root {
  /* Base colors (HSL format: H S% L%) */
  --background: 230 35% 10%;          /* Dark blue-gray */
  --foreground: 0 0% 95%;             /* Light text */
  
  /* Cards */
  --card: 230 35% 15%;
  --card-foreground: 0 0% 95%;
  
  /* Primary accent (your brand color) */
  --primary: 280 90% 60%;             /* Purple */
  --primary-foreground: 0 0% 100%;
  
  /* Secondary */
  --secondary: 230 35% 20%;
  --secondary-foreground: 0 0% 95%;
  
  /* Muted elements */
  --muted: 230 35% 25%;
  --muted-foreground: 230 15% 60%;
  
  /* Accent highlights */
  --accent: 340 90% 60%;              /* Pink */
  --accent-foreground: 0 0% 100%;
  
  /* Error states */
  --destructive: 0 85% 60%;
  --destructive-foreground: 0 0% 98%;
  
  /* Borders and inputs */
  --border: 230 35% 20%;
  --input: 230 35% 20%;
  --ring: 280 90% 60%;
  
  /* Border radius */
  --radius: 0.5rem;
  
  /* Charts */
  --chart-1: 280 90% 60%;
  --chart-2: 340 90% 60%;
  --chart-3: 200 90% 60%;
  --chart-4: 140 90% 60%;
  --chart-5: 40 90% 60%;
}
```

### Step 2: Register Your Theme

Edit `js/theme-manager.js`, add to the `themes` array:

```javascript
{ id: "myawesome", name: "✨ My Awesome", file: "myawesome.css" }
```

### Step 3: Test It!

1. Reload the application
2. Open theme dropdown
3. Select "✨ My Awesome"
4. Enjoy your custom theme!

## 🧩 Add a New Module

### Step 1: Create Module Files

Create directory: `mymodule/`

**mymodule/mymodule.html:**
```html
<div class="mymodule-layout">
  <h1>My Module</h1>
  <p>Hello from my custom module!</p>
</div>
```

**mymodule/mymodule.css:**
```css
.mymodule-layout {
  padding: var(--space-xl);
  max-width: 1200px;
  margin: 0 auto;
  padding-top: calc(var(--header-height) + var(--space-xl));
}

.mymodule-layout h1 {
  color: var(--text-primary);
  margin-bottom: var(--space-lg);
}
```

**mymodule/mymodule.js:**
```javascript
console.log('My Module loaded!');

// Initialize your module here
```

### Step 2: Register Module

Edit `js/layout.js`, add to `sections` object:

```javascript
const sections = {
  drive: "drive/drive.html",
  tasks: "tasks/tasks.html",
  mail: "mail/mail.html",
  chat: "chat/chat.html",
  mymodule: "mymodule/mymodule.html"  // Add this
};
```

### Step 3: Add to Apps Menu

Edit `index.html`, add to `.app-grid`:

```html
<a class="app-item" href="#mymodule" data-section="mymodule" role="menuitem">
  <div class="app-icon" aria-hidden="true">🚀</div>
  <span>My Module</span>
</a>
```

### Step 4: Test Your Module

1. Reload application
2. Click apps menu (9 dots icon)
3. Click "My Module"
4. See your module load!

## 🎯 Common Tasks

### Change Logo

Edit `index.html`, update `logo-icon`:

```html
<div class="logo-icon" style="background-image: url('path/to/logo.svg')"></div>
```

### Change App Title

Edit `index.html`:

```html
<title>My Awesome App</title>
<span class="logo-text">My Awesome App</span>
```

### Customize Colors in Code

```css
.my-component {
  /* Use theme variables */
  background: var(--primary-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.my-button {
  background: var(--accent-color);
  color: hsl(var(--primary-foreground));
}

.my-button:hover {
  background: var(--accent-hover);
}
```

### Add Custom Styles

Create `css/custom.css`:

```css
.my-custom-class {
  /* Your styles using theme variables */
  padding: var(--space-lg);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}
```

Link it in `index.html`:

```html
<link rel="stylesheet" href="css/app.css" />
<link rel="stylesheet" href="css/custom.css" />
```

## 🐛 Troubleshooting

### Theme Not Loading?

1. Check browser console (F12)
2. Verify file exists in `public/themes/`
3. Check CSS syntax (no commas in HSL values!)
4. Clear browser cache (Ctrl+Shift+R)

### Module Not Showing?

1. Check all three files exist (HTML, CSS, JS)
2. Verify registration in `layout.js`
3. Check browser console for errors
4. Ensure file paths are correct

### Colors Look Wrong?

HSL format is: `H S% L%` (no commas!)

**Wrong:** `hsl(280, 90%, 60%)`  
**Right:** `280 90% 60%`

### Can't Switch Sections?

1. Check console for JavaScript errors
2. Verify `window.switchSection` is defined
3. Try reloading the page
4. Clear localStorage

## 📚 Next Steps

- **Themes:** Read [THEMES.md](THEMES.md) for advanced theming
- **Components:** Check [COMPONENTS.md](COMPONENTS.md) for UI components
- **Full Docs:** See [README.md](README.md) for complete documentation

## 💡 Tips & Tricks

### Debug Mode

Open browser console and enable verbose logging:

```javascript
localStorage.setItem('debug', 'true');
location.reload();
```

### Test All Themes Quickly

```javascript
// Run in browser console
const themes = ThemeManager.getAvailableThemes();
themes.forEach((t, i) => {
  setTimeout(() => ThemeManager.loadTheme(t.id), i * 2000);
});
```

### HSL Color Picker

Use online tools:
- https://hslpicker.com/
- https://coolors.co/
- Chrome DevTools color picker

### Accessibility Check

```javascript
// Check contrast ratios in console
const bg = getComputedStyle(document.body).backgroundColor;
const fg = getComputedStyle(document.body).color;
console.log('Background:', bg);
console.log('Foreground:', fg);
```

## 🎓 Learning Resources

- **HSL Colors:** https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/hsl
- **CSS Variables:** https://developer.mozilla.org/en-US/docs/Web/CSS/--*
- **Accessibility:** https://www.w3.org/WAI/WCAG21/quickref/
- **Alpine.js:** https://alpinejs.dev/ (used in Drive/Tasks/Mail)

## 🤝 Get Help

- Check documentation files in this directory
- Review browser console for errors
- Test with different browsers
- Try disabling browser extensions

## ✅ Checklist for New Features

- [ ] Works with all themes
- [ ] Responsive on mobile
- [ ] Keyboard navigation works
- [ ] ARIA labels present
- [ ] Uses theme variables
- [ ] No console errors
- [ ] Documented in code
- [ ] Tested in multiple browsers

---

**Happy Building! 🚀**

*Need more help? See README.md, THEMES.md, and COMPONENTS.md*