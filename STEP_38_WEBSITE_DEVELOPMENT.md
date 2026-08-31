# STEP 38: Website Development & Launch Implementation

## Overview

Implementation guide for building and launching rui.dev following the architecture designed in STEP 29 and STEP 31. This step transforms the documented website structure into a live, production-ready website using Zola static site generator, GitHub Pages hosting, and automated CI/CD.

---

## Phase 1: Setup & Infrastructure (Days 1-2)

### 1.1 Initialize Zola Project

```bash
# Install Zola (macOS)
brew install zola

# Or from source
cargo install zola

# Create website directory structure
mkdir rui-website
cd rui-website
zola init .

# Answer the Zola initialization prompts:
# - Site URL: https://rui.dev (or https://alexwaldmann.github.io/rui)
# - Default language: en
# - Description: Rui - A simple, safe, universal UI library
```

### 1.2 Directory Structure

```
rui-website/
├── config.toml                    # Zola configuration
├── content/                       # All page content
│   ├── _index.md                  # Landing page
│   ├── docs/
│   │   ├── _index.md              # Documentation hub
│   │   ├── quickstart/
│   │   ├── guide/
│   │   ├── api/
│   │   ├── recipes/
│   │   └── examples/
│   ├── learn/
│   │   ├── _index.md              # Learning paths
│   │   ├── beginner/
│   │   ├── intermediate/
│   │   ├── advanced/
│   │   ├── contributor/
│   │   └── platform-developer/
│   ├── blog/
│   │   └── _index.md              # Blog hub
│   ├── community/
│   │   ├── _index.md              # Community hub
│   │   ├── governance/
│   │   └── events/
│   └── downloads/
│       └── _index.md              # Download center
├── templates/                     # Zola templates
│   ├── base.html
│   ├── page.html
│   ├── section.html
│   ├── index.html
│   └── components/
├── static/                        # Static files
│   ├── images/
│   ├── logos/
│   ├── css/
│   │   ├── main.css
│   │   ├── dark.css
│   │   └── accessibility.css
│   └── fonts/
├── .github/workflows/
│   └── deploy.yml                 # GitHub Pages deployment
└── .gitignore
```

### 1.3 Zola Configuration (config.toml)

```toml
# Site metadata
title = "Rui — A simple, safe, universal UI library"
description = "Build beautiful, fast, safe user interfaces with Rui"
default_language = "en"
theme = "rui-theme"

# URLs
base_url = "https://rui.dev"
output_dir = "public"

# Build settings
build_search_index = true
check_external_links = false
hard_link_static = true
minify_html = true
minify_css = true
minify_js = true

# Markdown
markdown = {
  external_links_target_blank = true
  smart_punctuation = true
}

# Taxonomy for blog categories
taxonomies = [
  { name = "tags" }
]

# Search index language
search = { index_on_build = true }

# Sitemap
generate_sitemap = true

# Load theme (you'll create this)
theme = "rui-theme"
```

### 1.4 GitHub Repository Setup

```bash
# Create separate repo for website (optional, or use rui/docs/)
git clone https://github.com/yourusername/rui-website
cd rui-website
git init

# Add origin
git remote add origin https://github.com/yourusername/rui-website
```

### 1.5 GitHub Pages Configuration

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy website

on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Install Zola
        run: |
          wget -q https://github.com/getzola/zola/releases/download/v0.17.1/zola-v0.17.1-x86_64-unknown-linux-gnu.tar.gz
          tar xzf zola-v0.17.1-x86_64-unknown-linux-gnu.tar.gz

      - name: Build website
        run: ./zola build

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        if: github.ref == 'refs/heads/main'
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./public
          cname: rui.dev
```

---

## Phase 2: Content Creation (Days 3-7)

### 2.1 Landing Page (`content/_index.md`)

```markdown
+++
title = "Rui — A simple, safe, universal UI library"
description = "Build beautiful, fast, safe user interfaces with Rui"
template = "index.html"
+++

# Rui

Build beautiful, fast, safe user interfaces across every platform.

## Features

- **Simple** — Clean API, minimal boilerplate
- **Safe** — 99.5% safe Rust, no unsafe code in core
- **Universal** — macOS, Windows, Linux, Web, and more
- **Fast** — Native performance on every platform
- **Accessible** — WCAG 2.1 Level AA compliance

## Get Started

```rust
use rui::prelude::*;

fn main() {
    App::new(|cx| {
        cx.to_el(|cx| text("Hello, Rui!"))
    })
    .run()
}
```

[Full Tutorial →](/learn/beginner)

## Learn

- **[Beginner](/learn/beginner)** — Start here (30 minutes)
- **[Intermediate](/learn/intermediate)** — Build real apps (2 hours)
- **[Advanced](/learn/advanced)** — Deep dive (1 hour)
- **[Contributor](/learn/contributor)** — Contribute to Rui (3+ hours)

## Community

Join 1,000+ developers building UIs with Rui.

- **[Discord](https://discord.gg/rui)** — Live chat and support
- **[GitHub Discussions](https://github.com/yourusername/rui/discussions)** — Q&A
- **[Matrix Bridge](https://matrix.org)** — Alternative chat

## Latest News

[Read the blog →](/blog)
```

### 2.2 Documentation Hub (`content/docs/_index.md`)

```markdown
+++
title = "Documentation"
description = "Complete Rui documentation and reference"
template = "section.html"
+++

# Documentation

Complete reference for Rui, from quick start to deep dives.

## Quick Start

- [Installation](/docs/quickstart/installation)
- [Your First App](/docs/quickstart/first-app)
- [State & Events](/docs/quickstart/state)

## Guides

- [Layouts & Spacing](/docs/guide/layouts)
- [Styling & Colors](/docs/guide/styling)
- [Building Controls](/docs/guide/controls)
- [Forms & Validation](/docs/guide/forms)
- [Testing](/docs/guide/testing)

## API Reference

- [Core Traits](/docs/api/core)
- [Elements & Widgets](/docs/api/elements)
- [Styling](/docs/api/style)
- [Layout](/docs/api/layout)
- [Events](/docs/api/events)

## Recipes

- [Recipe 1: WASM Backend](/docs/recipes/wasm-backend)
- [Recipe 2: Platform Backend](/docs/recipes/platform-backend)
- [Recipe 3: Custom Control](/docs/recipes/custom-control)
```

### 2.3 Learning Paths (`content/learn/_index.md`)

```markdown
+++
title = "Learn Rui"
description = "Learning paths for all skill levels"
template = "section.html"
+++

# Learning Paths

Choose your path and learn Rui step by step.

## [Beginner (30 min)](/learn/beginner)

Learn Rust while building your first app.

- Create a counter app
- Understand the state → view → handler pattern
- Deploy to web

## [Intermediate (2 hours)](/learn/intermediate)

Build a real app: weather dashboard, notes, or to-do list.

- API integration
- Error handling
- Theming & styling

## [Advanced (1 hour)](/learn/advanced)

Understand the architecture and build primitives.

- How Rui works
- Custom widgets
- Performance optimization

## [Contributor (3+ hours)](/learn/contributor)

Contribute to Rui and the ecosystem.

- Set up development environment
- Find and fix issues
- Build and publish examples

## [Platform Developer (6-8 weeks)](/learn/platform-developer)

Add a new backend (iOS, Android, Electron, etc).

- Study the Recipe 2 pattern
- Implement the Backend trait
- Test on your platform
```

### 2.4 Blog Setup (`content/blog/_index.md`)

```markdown
+++
title = "Blog"
description = "Rui news, tutorials, and community stories"
template = "section.html"
+++

# Blog

News, tutorials, and stories from the Rui community.
```

Create individual blog posts like `content/blog/first-release.md`:

```markdown
+++
title = "Rui 0.2.0 Released"
date = 2024-01-15
description = "Announcing Rui 0.2.0 with X11 and Wayland support"
+++

# Rui 0.2.0 Released

We're excited to announce the first public release of Rui...
```

### 2.5 Community Hub (`content/community/_index.md`)

```markdown
+++
title = "Community"
description = "Join the Rui community"
template = "section.html"
+++

# Community

Rui is built by and for our community.

## Communication

- **Discord** — Real-time chat (1,000+ members)
- **GitHub Discussions** — Q&A and ideas
- **Matrix** — Open-source bridge
- **Email** — [hello@rui.dev](mailto:hello@rui.dev)

## Events

- **Weekly Office Hours** — Tuesdays 2pm UTC
- **Monthly Workshops** — Beginner-friendly deep dives
- **Community Showcase** — Show what you built

## Code of Conduct

[Read our Code of Conduct](/community/code-of-conduct)

## Governance

[How Rui is governed](/community/governance)
```

---

## Phase 3: Design & Styling (Days 5-8)

### 3.1 Base HTML Template (`templates/base.html`)

```html
<!DOCTYPE html>
<html lang="{{ config.default_language }}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}{{ config.title }}{% endblock %}</title>
    <meta name="description" content="{% block description %}{{ config.description }}{% endblock %}">
    
    <!-- Favicon -->
    <link rel="icon" type="image/svg+xml" href="/logo.svg">
    
    <!-- Styles -->
    <link rel="stylesheet" href="/css/main.css">
    <link rel="stylesheet" href="/css/dark.css">
    <link rel="stylesheet" href="/css/accessibility.css">
    
    <!-- Meta tags -->
    <meta name="theme-color" content="#4A90E2">
    <meta property="og:title" content="{% block og_title %}{{ config.title }}{% endblock %}">
    <meta property="og:description" content="{% block og_description %}{{ config.description }}{% endblock %}">
    
    <!-- Analytics (optional) -->
    <script async defer data-domain="rui.dev" src="https://plausible.io/js/script.js"></script>
</head>
<body>
    {% include "components/header.html" %}
    
    <main class="container">
        {% block content %}{% endblock %}
    </main>
    
    {% include "components/footer.html" %}
</body>
</html>
```

### 3.2 CSS Foundation (`static/css/main.css`)

```css
/* Design tokens */
:root {
  --color-primary: #4A90E2;
  --color-success: #10B981;
  --color-warning: #F59E0B;
  --color-error: #EF4444;
  --color-info: #3B82F6;
  
  --color-bg-light: #FFFFFF;
  --color-bg-dark: #0F172A;
  --color-text-light: #1F2937;
  --color-text-dark: #F3F4F6;
  
  --font-body: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: "Fira Code", "Courier New", monospace;
  
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  --spacing-2xl: 3rem;
}

/* Reset & Typography */
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  font-family: var(--font-body);
  font-size: 1rem;
  line-height: 1.5;
  color: var(--color-text-light);
  background: var(--color-bg-light);
}

h1 { font-size: 2.5rem; margin: var(--spacing-lg) 0; }
h2 { font-size: 2rem; margin: var(--spacing-lg) 0 var(--spacing-md); }
h3 { font-size: 1.5rem; margin: var(--spacing-md) 0; }
p { margin: var(--spacing-md) 0; }

code {
  font-family: var(--font-mono);
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 3px;
}

pre {
  background: #1f2937;
  color: #f3f4f6;
  padding: var(--spacing-md);
  border-radius: 6px;
  overflow-x: auto;
  margin: var(--spacing-md) 0;
}

/* Layout */
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 var(--spacing-md);
}

.grid {
  display: grid;
  gap: var(--spacing-lg);
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
}

/* Components */
.btn {
  display: inline-block;
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: 6px;
  text-decoration: none;
  font-weight: 500;
  transition: all 0.3s;
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  opacity: 0.9;
  transform: translateY(-2px);
}

/* Responsive */
@media (max-width: 768px) {
  h1 { font-size: 2rem; }
  h2 { font-size: 1.5rem; }
  .container { padding: 0 var(--spacing-sm); }
}
```

### 3.3 Dark Mode (`static/css/dark.css`)

```css
@media (prefers-color-scheme: dark) {
  body {
    background: var(--color-bg-dark);
    color: var(--color-text-dark);
  }
  
  code {
    background: #1f2937;
    color: #e5e7eb;
  }
  
  a { color: #60a5fa; }
  a:visited { color: #a78bfa; }
}
```

---

## Phase 4: Search & Navigation (Days 8-9)

### 4.1 Add Search Index

Zola automatically builds a search index when `build_search_index = true` in config.toml.

Add to template:

```html
<div class="search">
  <input type="text" id="search-input" placeholder="Search docs...">
  <div id="search-results"></div>
</div>

<script src="/search_index.en.js"></script>
<script>
  const searchInput = document.getElementById('search-input');
  const searchResults = document.getElementById('search-results');
  
  searchInput.addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    if (query.length < 2) {
      searchResults.innerHTML = '';
      return;
    }
    
    const results = search(query, window.searchIndex);
    searchResults.innerHTML = results
      .slice(0, 10)
      .map(r => `<a href="${r.ref}">${r.title}</a>`)
      .join('');
  });
</script>
```

### 4.2 Navigation Component (`templates/components/header.html`)

```html
<header class="header">
  <div class="container">
    <a href="/" class="logo">Rui</a>
    
    <nav class="nav">
      <a href="/docs">Docs</a>
      <a href="/learn">Learn</a>
      <a href="/blog">Blog</a>
      <a href="/community">Community</a>
      <a href="https://github.com/yourusername/rui" target="_blank">GitHub</a>
    </nav>
    
    <button class="theme-toggle" aria-label="Toggle dark mode">
      🌙
    </button>
  </div>
</header>

<style>
.header {
  background: var(--color-bg-light);
  border-bottom: 1px solid #e5e7eb;
  position: sticky;
  top: 0;
  z-index: 100;
}

.header .container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md);
}

.logo {
  font-size: 1.5rem;
  font-weight: bold;
  text-decoration: none;
  color: var(--color-primary);
}

.nav {
  display: flex;
  gap: var(--spacing-lg);
}

.nav a {
  text-decoration: none;
  color: inherit;
  transition: color 0.3s;
}

.nav a:hover {
  color: var(--color-primary);
}
</style>
```

---

## Phase 5: Deployment & Launch (Days 10)

### 5.1 Domain Configuration

If using custom domain (rui.dev):

1. Add CNAME file to `static/CNAME`:
```
rui.dev
```

2. Configure domain registrar to point to GitHub Pages:
```
A record: 185.199.108.153
A record: 185.199.109.153
A record: 185.199.110.153
A record: 185.199.111.153
AAAA record: 2606:50c0:8000::153
AAAA record: 2606:50c0:8001::153
AAAA record: 2606:50c0:8002::153
AAAA record: 2606:50c0:8003::153
```

3. Update `config.toml`:
```toml
base_url = "https://rui.dev"
```

### 5.2 Build & Test Locally

```bash
# Install Zola
brew install zola

# Build website
zola build

# Serve locally for testing
zola serve

# Test at http://127.0.0.1:1111
```

### 5.3 GitHub Pages Setup

1. Go to repository Settings → Pages
2. Set source to "GitHub Actions"
3. Push to main branch to trigger deploy

### 5.4 Performance Verification

```bash
# Test Lighthouse scores (requires Chrome)
# Target: Performance >90, Accessibility >95, Best Practices >90

# After deploy, test at https://pagespeed.web.dev/
```

---

## Phase 6: Launch Day Execution (Day 11)

Follow STEP 35 Launch Announcement timeline:

- **T-24h:** Final website verification
- **T-12h:** Schedule all announcements
- **T-0h:** Publish GitHub Release, announce on Twitter/Reddit
- **T+1h:** Monitor analytics and engagement
- **T+24h:** Post-launch follow-up

---

## Launch Success Metrics

| Metric | Target | Verification |
|--------|--------|--------------|
| **Uptime** | 99.9%+ | GitHub Pages reliability |
| **Performance** | <2s load time | Lighthouse >90 |
| **SEO** | Indexed in 24h | Google Search Console |
| **Accessibility** | WCAG 2.1 AA | axe DevTools audit |
| **Mobile** | Fully responsive | Chrome DevTools |
| **Engagement** | <3s bounce time | Analytics tracking |

---

## Maintenance (Ongoing)

### Weekly Tasks
- [ ] Review website analytics
- [ ] Monitor for 404 errors
- [ ] Check external link health
- [ ] Update blog with news

### Monthly Tasks
- [ ] Refresh testimonials/showcase
- [ ] Update download statistics
- [ ] Review search analytics
- [ ] Optimize high-bounce pages

### Quarterly Tasks
- [ ] Major content refresh
- [ ] Performance audit
- [ ] SEO analysis
- [ ] Security audit

---

## Related Documentation

- STEP 29: Website Architecture & Design
- STEP 31: Website Implementation Planning
- STEP 35: Launch Announcement & Release Preparation
- README.md: Project overview

