# Scripts

Loading third-party scripts in Next.js.

## Use next/script

Always use `next/script` instead of native `<script>` tags for better performance.

```tsx
// Bad: Native script tag
<script src="https://example.com/script.js"></script>

// Good: Next.js Script component
import Script from 'next/script'

<Script src="https://example.com/script.js" />
```

## Inline Scripts Need ID

Inline scripts require an `id` attribute for Next.js to track them.

```tsx
// Bad: Missing id
<Script dangerouslySetInnerHTML={{ __html: 'console.log("hi")' }} />

// Good: Has id
<Script id="my-script" dangerouslySetInnerHTML={{ __html: 'console.log("hi")' }} />

// Good: Inline with id
<Script id="show-banner">
  {`document.getElementById('banner').classList.remove('hidden')`}
</Script>
```

## Don't Put Script in Head

`next/script` should not be placed inside `next/head`. It handles its own positioning.

```tsx
// Bad: Script inside Head
import Head from 'next/head'
import Script from 'next/script'

<Head>
  <Script src="/analytics.js" />
</Head>

// Good: Script outside Head
<Head>
  <title>Page</title>
</Head>
<Script src="/analytics.js" />
```

## Loading Strategies

```tsx
// afterInteractive (default) - Load after page is interactive
<Script src="/analytics.js" strategy="afterInteractive" />

// lazyOnload - Load during idle time
<Script src="/widget.js" strategy="lazyOnload" />

// beforeInteractive - Load before page is interactive (use sparingly)
// Only works in app/layout.tsx or pages/_document.js
<Script src="/critical.js" strategy="beforeInteractive" />

// worker - Load in web worker (experimental)
<Script src="/heavy.js" strategy="worker" />
```

## No Third-Party Analytics by Default

General Bots does not ship Google Analytics, Google Tag Manager, or any other
outbound analytics SDK. Do not add one to a bot or template unless the operator
explicitly asks for it and documents the data being sent. Prefer first-party,
same-origin measurements that stay on the operator's server.

## Other Third-Party Scripts

```tsx
// YouTube embed
import { YouTubeEmbed } from '@next/third-parties/google'

<YouTubeEmbed videoid="dQw4w9WgXcQ" />

// Google Maps
import { GoogleMapsEmbed } from '@next/third-parties/google'

<GoogleMapsEmbed
  apiKey="YOUR_API_KEY"
  mode="place"
  q="Brooklyn+Bridge,New+York,NY"
/>
```

## Quick Reference

| Pattern | Issue | Fix |
|---------|-------|-----|
| `<script src="...">` | No optimization | Use `next/script` |
| `<Script>` without id | Can't track inline scripts | Add `id` attribute |
| `<Script>` inside `<Head>` | Wrong placement | Move outside Head |
| Inline analytics scripts | Privacy risk | Do not add third-party analytics by default |
| `strategy="beforeInteractive"` outside layout | Won't work | Only use in root layout |
