# API Possibilities

Your bot can connect to hundreds of external services. This chapter shows what's possible when you integrate APIs into your bot conversations.

---

## How API Integration Works

When users talk to your bot, it can:

1. **Fetch information** from external services (weather, news, prices)
2. **Send data** to other systems (CRM, email, databases)
3. **Automate workflows** across multiple services

The bot decides when to call APIs based on what the user asks.

---

## What You Can Build

### Customer Service Bot
- Look up orders from your e-commerce system
- Check shipping status from carriers
- Create support tickets automatically

### Sales Assistant
- Pull customer info from your CRM
- Check inventory levels
- Generate quotes and send emails

### Personal Assistant
- Get weather forecasts
- Check calendar availability
- Book appointments

### Data Dashboard Bot
- Pull metrics from analytics platforms
- Get stock prices and crypto rates
- Aggregate data from multiple sources

---

## Popular API Categories

### 🌤️ Weather & Environment

| Service | What You Can Do |
|---------|-----------------|
| OpenWeatherMap | Current weather, forecasts, alerts |
| AirVisual | Air quality index by location |
| Open-Meteo | Free weather data worldwide |

**Example conversation:**

> **User:** What's the weather in São Paulo?
>
> **Bot:** It's 24°C and partly cloudy in São Paulo. Expect rain this afternoon.

---

### 💰 Finance & Payments

| Service | What You Can Do |
|---------|-----------------|
| Stripe | Process payments, manage subscriptions |
| CoinGecko | Crypto prices and market data |
| Exchange Rates | Currency conversion |

**Example conversation:**

> **User:** How much is Bitcoin right now?
>
> **Bot:** Bitcoin is currently $67,450 USD, up 2.3% today.

---

### 📰 News & Content

| Service | What You Can Do |
|---------|-----------------|
| NewsAPI | Headlines from 150+ sources |
| Hacker News | Tech news and discussions |
| Reddit | Posts from any subreddit |

**Example conversation:**

> **User:** What's trending in tech news?
>
> **Bot:** Top stories: Apple announces new M4 chip, OpenAI releases GPT-5...

---

### 📧 Communication

| Service | What You Can Do |
|---------|-----------------|
| SendGrid | Send transactional emails |
| Twilio | SMS messages and voice calls |
| Slack | Post messages to channels |

**Example conversation:**

> **User:** Send the report to the team
>
> **Bot:** Done! I've emailed the Q3 report to the sales team and posted a summary in #sales-updates.

---

### 🗺️ Maps & Location

| Service | What You Can Do |
|---------|-----------------|
| Google Maps | Directions, places, distances |
| OpenStreetMap | Free mapping data |
| IP Geolocation | Locate users by IP address |

**Example conversation:**

> **User:** How far is the airport?
>
> **Bot:** Guarulhos Airport is 25km away, about 45 minutes in current traffic.

---

### 🛒 E-Commerce

| Service | What You Can Do |
|---------|-----------------|
| Shopify | Manage products and orders |
| WooCommerce | WordPress store integration |
| Stripe | Payment processing |

**Example conversation:**

> **User:** Check my order status
>
> **Bot:** Order #12345 shipped yesterday via FedEx. Expected delivery: Friday.

---

### 📅 Productivity

| Service | What You Can Do |
|---------|-----------------|
| Google Calendar | Schedule events, check availability |
| Trello | Manage cards and boards |
| Notion | Create and update pages |

**Example conversation:**

> **User:** Schedule a meeting with John tomorrow at 2pm
>
> **Bot:** Done! I've added "Meeting with John" to your calendar for tomorrow at 2pm.

---

### 🎵 Entertainment

| Service | What You Can Do |
|---------|-----------------|
| Spotify | Search songs, manage playlists |
| YouTube | Search videos, get info |
| TMDB | Movie and TV show data |

**Example conversation:**

> **User:** What movies are playing this weekend?
>
> **Bot:** Here's what's in theaters: Dune Part 3, The Avengers 6...

---

### 🏥 Health & Fitness

| Service | What You Can Do |
|---------|-----------------|
| Fitbit | Activity and sleep data |
| Nutritionix | Food and nutrition info |
| Strava | Running and cycling stats |

**Example conversation:**

> **User:** How many calories in a Big Mac?
>
> **Bot:** A Big Mac has 563 calories, 33g fat, and 25g protein.

---

### 🔐 Authentication

| Service | What You Can Do |
|---------|-----------------|
| Auth0 | User login and registration |
| Firebase Auth | Google, Facebook, email login |
| Have I Been Pwned | Check for data breaches |

---

### 🛠️ Developer Tools

| Service | What You Can Do |
|---------|-----------------|
| GitHub | Repo info, issues, PRs |
| Jira | Create and manage tickets |
| Sentry | Error tracking and alerts |

**Example conversation:**

> **User:** Create a bug ticket for the login issue
>
> **Bot:** Created PROJ-456: "Login button not responding on mobile" - assigned to dev team.

---

## API Marketplaces

Find more APIs at:

- **RapidAPI** - 40,000+ APIs in one place
- **APILayer** - Curated collection of useful APIs
- **Public APIs** - Free API directory

---

## Things to Consider

### Rate Limits
Most APIs limit how many requests you can make. Plan for this in high-traffic bots.

### API Keys
Keep your API keys secure. Never expose them in client-side code.

### Costs
Many APIs are free up to a limit, then charge per request. Monitor your usage.

### Reliability
Have fallback responses when APIs are slow or unavailable.

---

## Getting Started

1. **Choose an API** that matches what your bot needs to do
2. **Get API credentials** (usually free to sign up)
3. **Create a tool** in your `.gbdialog` folder that calls the API
4. **Test it** by asking your bot questions that trigger the API

Your bot's LLM automatically learns when to use each tool based on what users ask.

---

## See Also

- [Keywords Reference](./keywords.md) - BASIC commands for API calls
- [HTTP Operations](./keywords-http.md) - GET, POST, PUT, PATCH, DELETE
- [Tools and Integration](../08-rest-api-tools/README.md) - Building custom tools