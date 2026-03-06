# QA Plan: Fix Scrollbar Styling

## Verification Steps

### 1. Code review
- [ ] `index.css` contains `scrollbar-width: thin` and `scrollbar-color` on `*`
- [ ] `index.css` contains `::-webkit-scrollbar { width: 6px; height: 6px; }`
- [ ] `::-webkit-scrollbar-track` uses `background: transparent`
- [ ] `::-webkit-scrollbar-thumb` uses a dark oklch color with border-radius
- [ ] `::-webkit-scrollbar-thumb:hover` uses a slightly lighter color
- [ ] `::-webkit-scrollbar-corner` uses `background: transparent`
- [ ] No other files modified (global CSS rules cover all surfaces)

### 2. Visual browser check (Webkit/Blink)
- [ ] Open the app in Chrome or Safari
- [ ] Scroll in sidebar → thin dark scrollbar visible, no native OS chrome
- [ ] Scroll in a panel/modal → same styled scrollbar
- [ ] Scrollbar thumb color matches dark theme palette

### 3. Visual browser check (Firefox)
- [ ] Open the app in Firefox
- [ ] Scroll in sidebar → thin scrollbar visible
- [ ] No native-width scrollbar appears

### 4. No regressions
- [ ] Existing layout not broken (no unexpected overflow hiding content)
- [ ] `npm run build` passes without errors
