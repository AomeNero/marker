# Distribution and Launch Checklist

Last researched: 2026-07-22

## Current Baseline

- GitHub: 710 stars, 38 forks
- GitHub release asset downloads: 9,520 total
- GitHub traffic (rolling 14-day window at capture): 6,000 views / 2,086 unique visitors
- Official website: live at `https://marker.cn/`
- Microsoft Store / WinGet: live
- Scoop Extras: live
- Existing community posts: V2EX, Reddit r/tauri, Reddit r/vuejs
- New technical article: Juejin submission is live at `https://juejin.cn/spost/7664877035786600458` and awaiting platform review
- Missing high-intent listings: Product Hunt, Show HN, AlternativeTo, Homebrew Cask

Metrics are a dated baseline, not evergreen marketing copy. Release asset downloads count files, not unique users.

## Ready Now

### GitHub Pages Website

- Status: live
- Local source: `docs/`
- Workflow: `.github/workflows/pages.yml`
- Custom domain: `https://marker.cn/`
- Legacy URL: `https://ifer47.github.io/marker/`
- SEO/social metadata: canonical URLs, absolute Open Graph image, Twitter card metadata, SoftwareApplication + FAQ structured data
- Remaining manual work: submit `https://marker.cn/sitemap.xml` in Google Search Console and Bing Webmaster Tools

### WinGet via Microsoft Store

Marker already appears in the Microsoft Store WinGet source:

```powershell
winget install --id 9N6623X973JV --source msstore --accept-source-agreements
```

Keep this as the primary Windows package-manager install path for now.

Official references:

- Windows Package Manager overview: https://learn.microsoft.com/en-us/windows/package-manager/
- Submit packages: https://learn.microsoft.com/en-us/windows/package-manager/package/
- Submit manifests to repository: https://learn.microsoft.com/en-us/windows/package-manager/package/repository

### Product Hunt

Use the draft in `promo/launch-kit.md`.

Official notes checked:

- Product Hunt says makers can post their own product and start by clicking the Post button after logging in.
- Product Hunt recommends kicking off the conversation with a maker comment after posting.
- Product Hunt account age restrictions may apply.

Links:

- Post flow help: https://help.producthunt.com/en/articles/479557-how-to-post-a-product
- Getting started: https://help.producthunt.com/en/articles/2305333-getting-started
- Featuring guidelines: https://help.producthunt.com/en/articles/9883485-product-hunt-featuring-guidelines

### Show HN

Use the draft in `promo/launch-kit.md`.

Important rule: submit something users can actually try, not a landing page alone. Use the GitHub repo or latest release as the URL.

Links:

- Guidelines: https://news.ycombinator.com/showhn.html
- Submit: https://news.ycombinator.com/submit

### AlternativeTo

Use this metadata:

- Name: `Marker`
- Platforms: Windows, macOS
- License: Open Source / MIT
- Tags: screen annotation, desktop annotation, whiteboard, drawing, productivity, presentation
- Website: GitHub repo or GitHub Pages website
- Description: use the short description from `promo/launch-kit.md`
- Alternatives to list against: ZoomIt, Epic Pen, gInk, ppInk, ScreenBrush

Official FAQ says to use "Suggest new application" from the user menu and submit platforms, license, descriptions, tags, and related fields.

Link: https://alternativeto.net/faq/

## Package Manager Expansion

### Scoop

Status: live in Scoop Extras.

- Manifest: https://github.com/ScoopInstaller/Extras/blob/master/bucket/marker.json
- Initial PR: https://github.com/ScoopInstaller/Extras/pull/18304
- Install: `scoop bucket add extras && scoop install marker`

The upstream manifest has automatic GitHub version checks. Monitor its update PR after each release; do not open duplicate update PRs.

Local note: `packaging/scoop/README.md`

Target repository:

- Scoop Extras: https://github.com/ScoopInstaller/Extras

### Homebrew Cask

Draft cask exists at `packaging/homebrew/marker.rb`.

Before submitting, test on macOS:

```bash
brew install --cask ./packaging/homebrew/marker.rb
brew uninstall --cask marker
```

Official references:

- Adding software: https://docs.brew.sh/Adding-Software-to-Homebrew
- Cask cookbook: https://docs.brew.sh/Cask-Cookbook
- Acceptable casks: https://docs.brew.sh/Acceptable-Casks

## Optional Directories

### SourceForge

Useful if you want an additional open-source project page and download mirror. SourceForge expects an open-source project, summary, icon, description, and screenshots.

Links:

- Create project: https://sourceforge.net/p/forge/documentation/Create%20a%20New%20Project/
- Promote project: https://sourceforge.net/p/forge/documentation/Promoting%20your%20project/

### Softpedia

Potentially useful for discovery, but submit carefully and make the GitHub/Microsoft Store official channel clear. Prioritize directories that link to the official downloads without repackaging.

## Existing Coverage — Do Not Duplicate

- V2EX: https://www.v2ex.com/t/1204012
- V2EX follow-up discovered in search: https://www.v2ex.com/t/1222261
- Reddit r/tauri: https://www.reddit.com/r/tauri/comments/1sjh08d/marker_a_lightweight_screen_annotation_tool/
- Reddit r/vuejs: https://www.reddit.com/r/vuejs/comments/1slvt7f/marker_a_lightweight_screen_annotation_tool/
- Third-party Chinese download page found in search: https://www.cr173.com/soft/1668222.html (outdated v1.0.0; do not endorse its repackaged download)
- Organic article: https://myqqjd.com/84790.html

Only post again in an existing community for a substantial feature story, tutorial, or milestone. Do not repost minor patch releases.

## 30-Day Launch Order

### Week 1 — Conversion foundation

1. Deploy the corrected website metadata and evergreen copy.
2. Verify the public release and all official downloads.
3. Capture one 10–15 second click-through demo and one whiteboard demo.
4. Submit AlternativeTo and Product Hunt with the website as the canonical link.
5. Submit Show HN with the GitHub repository as the URL and a technical maker comment.

### Week 2 — Chinese creator channels

1. Publish one Bilibili demo video and reuse the vertical cut for 视频号 / 抖音 / 小红书.
2. Published a transparent “作者自荐” Zhihu answer comparing Marker with ZoomIt and gInk: https://www.zhihu.com/question/384946892/answer/2063224352896766963
3. Publish the Tauri implementation article on 掘金; optionally cross-post to CSDN / OSCHINA with a canonical link back to the website or GitHub.
4. Pitch 少数派 only after the demo video and current screenshots are ready.

### Week 3 — Open-source distribution

1. Confirm the Scoop manifest auto-updates after the latest release.
2. Test the Homebrew cask on both Apple Silicon and Intel if possible, then submit.
3. Submit to relevant curated awesome lists only where Marker clearly matches the list scope.
4. Post to r/opensource with an open-source / architecture angle; do not reuse the r/tauri copy.

### Week 4 — Earned media and compounding SEO

1. Send personalized pitches to 10 software newsletters / teaching-tech editors.
2. Publish “Marker vs ZoomIt vs Epic Pen” and “How to annotate over any app” articles on `marker.cn`.
3. Answer relevant Zhihu, Reddit, and Stack Exchange questions with disclosed affiliation and useful comparison content.
4. Review UTM traffic, stars, downloads, and retention; repeat the two channels with the strongest qualified traffic.

## Channel Priority Matrix

| Priority | Channel | Best angle | Required asset | Status |
| :--- | :--- | :--- | :--- | :--- |
| P0 | Product Hunt | tiny open-source app + click-through | 3–5 images, maker comment | Not submitted |
| P0 | Show HN | transparent overlay architecture | repo URL, technical post | Not submitted |
| P0 | AlternativeTo | open-source Epic Pen / ZoomIt alternative | listing metadata | Not submitted |
| P0 | Bilibili | 15-second before/after demo | horizontal video | Not published |
| Live | 知乎 | disclosed comparison answer | workflow + honest comparison | Published 2026-07-22; self-recommendation distributing |
| Live | Scoop Extras | Windows package install | upstream manifest | Published; automatic update checks enabled |
| P1 | Homebrew Cask | macOS package install | tested cask | Draft exists; macOS test pending |
| P1 | r/opensource | local-first MIT project | tailored post | Not posted |
| P1 | 掘金 / dev.to | Tauri overlay implementation | technical article | 掘金 submitted (review pending); dev.to draft ready |
| P1 | 少数派 | real teaching/demo workflow | polished screenshots + video | Pitch pending |
| P2 | SourceForge | open-source directory presence | project page assets | Optional |
| P2 | Softpedia | software discovery | official-link-only submission | Optional |

## Success Metrics

Track by channel at 24 hours, 7 days, and 30 days:

- Qualified website sessions (UTM)
- GitHub stars and watchers
- Release asset downloads (remember: per-file, not users)
- Microsoft Store acquisitions if Partner Center is available
- Issue quality and first-time contributors
- Video completion rate and saves/shares
- Conversion from landing page to an official download

Avoid optimizing for raw impressions alone; channels that produce downloads, useful feedback, or repeat users matter more.
