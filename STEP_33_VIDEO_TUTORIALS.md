# STEP 33: Video Tutorials & Content Strategy

## Overview

Create a library of high-quality video tutorials and recorded walkthroughs to complement written documentation. Videos accelerate learning and increase engagement.

**Goal:** Produce 10-12 tutorial videos covering learning paths from beginner to platform developer, suitable for YouTube, website embedding, and offline distribution.

---

## Content Strategy

### Video Format & Production

**Target platforms:**
- YouTube channel (primary distribution)
- Embedded on rui.dev (learning paths)
- Offline downloads (MP4 for workshops)
- Subtitled/captioned (accessibility)

**Production approach:**
- Screen recording + voiceover
- Terminal/editor screen capture (1440p)
- Code examples with syntax highlighting
- Animated diagrams for architecture concepts
- Captions for accessibility (auto-generate + review)

**Tools:**
- **Recording:** OBS Studio (free, open-source)
- **Editing:** DaVinci Resolve (free, professional-grade)
- **Audio:** Audacity (free, audio editing)
- **Hosting:** YouTube (free with custom domain linking via rui.dev)

### Target Audience & Topics

#### Beginner Path (4 videos, 45 min total)

**Video 1: "Hello rui" (8 min)**
- What is rui? (declarative UI, Rust, cross-platform)
- Install Rust and create project
- Copy-paste hello-world example
- Run app and explore
- Call-to-action: "Subscribe for the next tutorial"

**Video 2: "Understanding State" (12 min)**
- State = data
- View = function of state
- Handlers = update state
- Counter example (increment/decrement)
- Edit state and see UI update
- Call-to-action: "Try it yourself"

**Video 3: "Layouts & Spacing" (12 min)**
- Rows and columns
- Gaps and padding
- Alignment and justification
- Real app: Simple form (name, email, submit)
- Layout debugging (showing structure)
- Call-to-action: "Share your layout on GitHub Discussions"

**Video 4: "Styling & Colors" (13 min)**
- Color roles (Tone::Accent, Tone::Success, etc.)
- Light and dark modes
- Custom styling with `.fill()`, `.pad()`, `.border()`
- Theme switching
- Real app: Theme-aware card component
- Call-to-action: "Build your first custom widget"

#### Intermediate Path (4 videos, 60 min total)

**Video 5: "Building Custom Controls" (15 min)**
- Checkbox exemplar (copy-modify pattern)
- State → View → Handler pattern
- Testing with Harness
- Deploy to desktop
- Common mistakes and debugging
- Call-to-action: "Build a toggle, radio button, or custom input"

**Video 6: "Real App: Weather Dashboard" (18 min)**
- Project structure (state, view, handlers)
- API integration (fetch weather data)
- Error handling (failed requests)
- Layouts (main panel, forecast cards, refresh button)
- Styling (weather-appropriate colors)
- Call-to-action: "Modify for your city"

**Video 7: "Testing & Quality" (14 min)**
- Unit tests (Harness framework)
- Integration tests
- Running tests locally
- CI/CD (GitHub Actions)
- Code coverage
- Call-to-action: "Write tests for your app"

**Video 8: "Deployment & Release" (13 min)**
- Building for release (`cargo build --release`)
- Platform-specific considerations
- Code signing (macOS/Windows)
- App stores (if applicable)
- Version management
- Call-to-action: "Ship your first app"

#### Advanced Path (2 videos, 40 min total)

**Video 9: "Performance Optimization" (20 min)**
- Profiling tools (Xcode Instruments, DevTools)
- Common bottlenecks (recomputing views, large lists)
- Optimization techniques (memoization, caching)
- Animation performance (60fps target)
- Real-world example: Optimizing a large list
- Call-to-action: "Profile your app and fix slow spots"

**Video 10: "Building Custom Primitives" (20 min)**
- Understanding the draw API
- Painter abstraction (fill, stroke, text)
- Responsive custom widget example (drawing editor)
- Handling complex interactions (drag, pan, zoom)
- Real-world example: Simple paint brush
- Call-to-action: "Build your own drawing app"

#### Platform Developer Path (2 videos, 50 min total)

**Video 11: "Recipe 2: Implementing a Backend" (30 min)**
- Why backends matter (reach new platforms)
- Anatomy of the Backend trait (6 methods)
- Study case: X11 backend walkthrough
- Coordinate contracts (DPI scaling, event translation)
- Testing strategy (parity verification)
- Call-to-action: "Design a backend for your platform"

**Video 12: "From Idea to Merged PR" (20 min)**
- Contributing workflow (fork, clone, branch, commit)
- Writing good commit messages
- Creating a pull request
- Code review feedback
- Getting merged and recognized
- Real example: From issue to merged PR
- Call-to-action: "Make your first contribution"

---

## Production Workflow

### Pre-Production (Planning)

**For each video:**
1. Write script (200-300 words, 5-8 min reading time)
2. Create storyboard (key scenes, transitions)
3. Prepare code examples (test locally first)
4. Set up screen (desktop size, font size, theme)
5. Record audio (voiceover, edit)
6. Create thumbnail and title

**Example script (Video 1: "Hello rui"):**

```
[INTRO]
"Hello! I'm [name], and in this video, I'll show you how to build your
first app with rui, a declarative UI library for Rust that runs on
macOS, Windows, Linux, and the web.

By the end of this video, you'll have a working app running on your
machine. Let's start."

[INSTALL RUST]
"First, we need Rust. If you don't have it, visit rustup.rs and
follow the instructions. It takes about 5 minutes."

[SHOW: rustup.rs in browser, install command]

"Once installed, verify with: rustc --version"

[SHOW: rustc --version in terminal]

"Great! Now let's create a new project."

[CREATE PROJECT]
cargo new hello_rui
cd hello_rui

"The default project looks like this..."

[SHOW: code editor with main.rs]

"Let's delete this and paste a rui example instead."

[SHOW: Replacing code]

"Now build and run:"

cargo run

"And... there's our app! You can close it by clicking the window close button."

[OUTRO]
"That's it! In the next video, we'll explore state and how to make
your app interactive. Subscribe for more. Thanks for watching!"
```

### Recording

**Desktop setup:**
```
Resolution: 1440p (wide-aspect)
Font size: 18-20pt (readable on YouTube)
Theme: Light mode (better video visibility)
Terminal: Clear background, sans-serif font
Editor: Code with syntax highlighting enabled
```

**Recording checklist:**
- [ ] Close unnecessary windows
- [ ] Disable notifications
- [ ] Set audio to 44.1kHz, stereo
- [ ] Test microphone levels (-6dB peak)
- [ ] Start recording 3 seconds early (silence)
- [ ] Speak clearly at moderate pace (not too fast)
- [ ] Pause between major points
- [ ] End recording 2 seconds after outro (silence)

**OBS Studio Configuration:**

```ini
[Output]
SimpleOutput.RecFormat=mp4
SimpleOutput.RecQuality=high
SimpleOutput.OutputPath=/path/to/recordings

[Video]
BaseCX=2560
BaseCY=1440
OutputCX=1440
OutputCY=810
FPS=30
```

### Post-Production (Editing)

**For each video:**

1. **Import & organize** (DaVinci Resolve)
   - Import video file (MP4)
   - Import audio (WAV if separate)
   - Organize on timeline

2. **Cut & edit**
   - Remove pauses/stutters
   - Add transitions (fade, cut)
   - Adjust audio levels (normalize to -6dB)

3. **Graphics & captions**
   - Add title card (3 sec)
   - Add captions (auto-generate, then review)
   - Add code overlays (if useful)
   - Add channel watermark/logo (optional)

4. **Color & effects**
   - Normalize color (if lighting inconsistent)
   - Adjust contrast/saturation (if needed)
   - Add background music (optional, low volume)

5. **Export**
   - Export as MP4 (H.264, 1440p, AAC audio)
   - Target bitrate: 5-10 Mbps

6. **Subtitle generation**
   - Use auto-captions (YouTube or other tool)
   - Manual review and correction (15 min per video)
   - Export SRT file for website embedding

### Quality Assurance

**Before publishing:**
- [ ] Audio is clear (no background noise)
- [ ] Code examples work (verified before recording)
- [ ] Pace is reasonable (not too fast/slow)
- [ ] Captions are accurate and complete
- [ ] Thumbnail is professional and readable
- [ ] Title is descriptive and searchable
- [ ] Description includes timestamps and links
- [ ] No dead links or outdated information

---

## YouTube Channel Setup

### Channel Details

**Name:** rui — A Declarative UI Library for Rust

**Description:**
```
Build beautiful, native UIs in Rust. Cross-platform: macOS, Windows,
Linux, and the web. Zero dependencies, strong types, immediate-mode.

Tutorials, deep dives, and examples for all skill levels.

🔗 Website: https://rui.dev
📚 Documentation: https://rui.dev/docs
🐙 GitHub: https://github.com/...
💬 Discussions: https://github.com/.../discussions
```

**Channel Art:**
- Banner: 2560×1440px (rui logo + tagline)
- Avatar: Logo icon (512×512px)
- Theme: Colors from STEP 28 branding

### Playlist Organization

**Playlists:**

1. **Beginner** (4 videos) — "Get Started with rui"
2. **Intermediate** (4 videos) — "Build Real Apps"
3. **Advanced** (2 videos) — "Master rui"
4. **Platform Dev** (2 videos) — "Extend rui"
5. **Archive** (older/bonus videos)

### Video Metadata

**Format for each video:**

**Title:** "[Level] Topic — Brief description" (50-60 chars)
- Example: "Beginner 2: Understanding State — Counter app in 12 minutes"

**Description:** (200-300 words)
```
Learn about state in rui — the foundation of reactive UI.

In this video, we'll build a counter app and explore how state flows
to the view and handlers update it.

⏱️ Timestamps:
0:00 Intro
1:30 State definition
4:00 View function
7:15 Handlers
10:00 Demo
11:30 Next steps

📚 Resources:
- Guide: https://rui.dev/docs/guide/state/
- Examples: https://rui.dev/docs/examples/
- Recipes: https://rui.dev/docs/recipes/

💬 Questions? Join the discussion: https://github.com/.../discussions

🔔 Subscribe for more tutorials!
```

**Tags:** (10-15 tags per video)
- rui
- rust
- ui library
- declarative ui
- cross-platform
- state management
- rust tutorial
- gui
- immediate-mode
- [level: beginner/intermediate/advanced]

**Thumbnail:**
- Text overlay: "Hello rui" or "State?" or key concept
- Colors from brand palette
- High contrast, readable at small size
- Consistent style across series

---

## Embedding on Website

### Learning Path Pages

**Example: docs/learn/beginner/_index.md**

```markdown
# Beginner Path — Get Started with rui

Learn the fundamentals of building UIs with rui. This path is designed
for developers new to Rust or declarative UI patterns.

## Videos

### 1. Hello rui (8 min)

<div style="position: relative; width: 100%; padding-bottom: 56.25%;">
  <iframe style="position: absolute; top: 0; left: 0; width: 100%; height: 100%;" 
    src="https://www.youtube-nocookie.com/embed/VIDEO_ID_1" 
    frameborder="0" allow="accelerometer; autoplay; clipboard-write; 
    encrypted-media; gyroscope; picture-in-picture" allowfullscreen>
  </iframe>
</div>

**What you'll learn:**
- What is rui and why use it?
- Install Rust
- Create and run your first app

**Code example:**
```rust
fn main() {
    rui::app().run()
}
```

[Continue to Video 2 →](#video-2)
```

### Offline Distribution

Create a downloadable "Video Pack" (MP4 files):

```
rui-video-pack-v0.2.0/
├── README.md
├── beginner/
│   ├── 01-hello-rui.mp4
│   ├── 02-understanding-state.mp4
│   ├── 03-layouts.mp4
│   └── 04-styling.mp4
├── intermediate/
│   ├── 05-custom-controls.mp4
│   ├── 06-weather-app.mp4
│   ├── 07-testing.mp4
│   └── 08-deployment.mp4
├── advanced/
│   ├── 09-performance.mp4
│   └── 10-primitives.mp4
└── platform-dev/
    ├── 11-backends.mp4
    └── 12-contributing.mp4
```

Release as ZIP download on GitHub Releases (for offline viewing, workshops).

---

## Analytics & Engagement

### Tracking Metrics

**YouTube Analytics:**
- Views, watch time, average view duration
- Viewer retention (where do people stop watching?)
- Click-through rate (CTR) to website/GitHub
- Engagement (likes, comments, shares)

**Website Integration:**
- Video play count (if embedded)
- Time on page (learning path pages)
- Conversion to "Get Started"

**Success targets (first 3 months):**
- 500+ total views across all videos
- 100+ subscribers
- 50% average view duration (viewers watch at least half)
- 5% CTR to website

### Engagement Strategy

**Comments & Community:**
- Reply to every comment (first week)
- Pin helpful comments
- Respond to questions with timestamps
- Foster discussion in GitHub Discussions (link in description)

**Community-Generated Content:**
- Feature viewer projects (monthly "Community Spotlight")
- Share interesting examples from discussions
- Encourage sharing on Twitter/LinkedIn with #rui

**Call-to-Actions:**
- Subscribe (start and end of every video)
- Join GitHub Discussions (description)
- Try the code (in description)
- Share feedback (in comments)

---

## Production Timeline

### Phase 1: Foundation (Weeks 1-2)

- [ ] Set up recording environment (OBS Studio, microphone test)
- [ ] Set up editing environment (DaVinci Resolve, templates)
- [ ] Write scripts for Beginner videos (4 scripts, ~1000 words)
- [ ] Record Beginner videos (4 × 1-2 hours, including retakes)
- [ ] Edit and subtitle Beginner videos
- [ ] Create thumbnails and titles
- [ ] Set up YouTube channel

### Phase 2: Core Content (Weeks 3-5)

- [ ] Write scripts for Intermediate videos (4 scripts)
- [ ] Record Intermediate videos
- [ ] Edit and subtitle
- [ ] Publish Beginner series (1 per week)

### Phase 3: Advanced Content (Weeks 6-7)

- [ ] Write scripts for Advanced + Platform Dev videos (4 scripts)
- [ ] Record videos
- [ ] Edit and subtitle
- [ ] Publish Intermediate series (1 per week)

### Phase 4: Polish & Launch (Week 8)

- [ ] Publish Advanced/Platform Dev videos
- [ ] Create playlist pages on website
- [ ] Set up embedded players
- [ ] Create offline video pack
- [ ] Launch announcement

**Total: 8 weeks**

---

## Success Criteria

### Content Quality

- [ ] 10-12 videos completed and published
- [ ] All videos have accurate captions
- [ ] Audio is clear and professional
- [ ] Code examples are correct and tested
- [ ] Pacing is appropriate for skill level
- [ ] Production value matches YouTube standards

### Engagement

- [ ] YouTube channel with 100+ subscribers (month 1)
- [ ] Average view duration > 50%
- [ ] 5-10 comments per video (community engagement)
- [ ] 5%+ click-through to website/GitHub
- [ ] 500+ total views (month 1)

### Integration

- [ ] All videos embedded on rui.dev/learn/
- [ ] Offline video pack available on GitHub Releases
- [ ] Links in descriptions point to docs and GitHub
- [ ] Playlists organized by skill level

### Accessibility

- [ ] All videos captioned (100% coverage)
- [ ] Captions accurate (spot-checked)
- [ ] Audio levels normalized (-6dB peak)
- [ ] Color-blind friendly (captions cover visual info)

---

## Next Steps

1. **STEP 33A:** Write scripts and prepare examples
2. **STEP 33B:** Record Beginner series (4 videos)
3. **STEP 33C:** Edit, caption, publish Beginner series
4. **STEP 33D:** Record Intermediate series
5. **STEP 33E:** Record Advanced + Platform Dev series
6. **STEP 34:** Community growth (Discord, workshops, events)
7. **STEP 35:** Launch announcement and marketing push

---

## Integration with STEP 30 Launch Checklist

This STEP 33 deliverable feeds into STEP 30's checklist:

- ✅ Learning & engagement → 10-12 tutorial videos for all skill levels
- ✅ Accessibility → Captions for all videos, audio-described graphics
- ✅ Community engagement → Encourages interaction via comments/discussions
- ✅ Metrics → Tracks views, engagement, conversion to GitHub

Once STEP 33 is complete, users have multiple ways to learn:
- Written guides (CLAUDE.md, docs/)
- Learning paths (STEP 24)
- Video tutorials (STEP 33)
- Real examples (STEP 26)
- API reference (STEP 32)

This comprehensive, multi-format approach maximizes accessibility and engagement across different learning styles.
