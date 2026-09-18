# Sloth — Design Document

> **Version:** 1.0.0  
> **Last Updated:** 2026-09-18  

---

## 1. Design Philosophy

Sloth is a terminal application — it lives in the environment the user already loves. The design must feel like it **belongs** in a sophisticated terminal setup, not like a port of a mobile app. Every pixel (character) counts.

### Core Design Principles

1. **Information Density** — Show as much useful information as possible without clutter. Terminal users are comfortable with dense UIs.
2. **Speed Feedback** — The user must always know something is happening. Loading indicators everywhere, instant key response, never hang silently.
3. **Vim Intuition** — `j/k` navigation, `/` to search, `gg/G` for top/bottom. Power users know this muscle memory.
4. **Contextual** — The bottom status bar and keybinding hints change based on where the cursor is.
5. **Themeable** — The entire palette is swappable. No hardcoded colors anywhere in render code.

---

## 2. Visual Language

### 2.1 Typography

Since terminals use monospace fonts, visual hierarchy comes from:
- **Width** — full-width titles, narrower metadata
- **Style** — Bold for titles, dim for metadata, italic for descriptions
- **Icons** — Unicode symbols and emoji for instant visual recognition
- **Spacing** — Careful padding to create breathing room

### 2.2 Icon System

Consistent icons throughout the UI:

| Context | Icon | Meaning |
|---|---|---|
| Content type: Movie | `🎬` | Movie |
| Content type: Series | `📺` | TV Series |
| Content type: Anime | `⛩` | Anime |
| Content type: Sport | `⚽` | Live Sport |
| Content type: F1 | `🏎` | Formula 1 |
| Content type: TV | `📡` | Live TV |
| Status: Live | `●` (red, blinking) | Currently live |
| Status: Upcoming | `⏰` | Scheduled |
| Status: Finished | `✓` | Completed |
| Status: Loading | `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` (spinner) | Loading |
| Action: Play | `▶` | Play |
| Action: Download | `⬇` | Download |
| Action: Favorite | `♥` / `♡` | Favorite on/off |
| Quality: 4K | `4K` | Ultra HD |
| Quality: 1080p | `HD` | Full HD |
| Quality: 720p | `HD` | HD |
| Quality: 480p | `SD` | Standard |
| Sub/Dub: Sub | `[Sub]` | Subtitled |
| Sub/Dub: Dub | `[Dub]` | Dubbed |
| F1 Session | `🏁` Race, `⏱` Quali, `⚡` Sprint | Session type |
| AniList: Current | `▶ Watching` | AniList status |
| AniList: Plan | `📋 Plan` | AniList status |
| AniList: Completed | `✓ Complete` | AniList status |
| Rating | `★ 8.4` | Rating display |

### 2.3 Color Usage (Catppuccin Mocha reference)

| Element | Color Token | Hex (Mocha) |
|---|---|---|
| Background | `bg` | `#1e1e2e` |
| Elevated panels | `bg_elevated` | `#242436` |
| Borders | `border` | `#585b70` |
| Focused border | `border_focus` | `#89b4fa` |
| Primary text | `text` | `#cdd6f4` |
| Dimmed text | `text_dim` | `#a6adc8` |
| Muted/hint text | `text_muted` | `#6c7086` |
| Accent (links, selected) | `accent` | `#89b4fa` |
| Success / Watched | `success` | `#a6e3a1` |
| Warning / Upcoming | `warning` | `#f9e2af` |
| Error / Failed | `error` | `#f38ba8` |
| Live indicator | `error` | `#f38ba8` (red) |
| Info / Loading | `info` | `#89dceb` |
| Active tab | `tab_active` | `#89b4fa` |
| Inactive tab | `tab_inactive` | `#6c7086` |

---

## 3. Layout System

### 3.1 Global Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ SLOTH ║ Movies │ Anime │ Sports │ F1 │ Live TV │ ≡ More        │  <- Tab bar (3 rows)
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│                      [Main Content Area]                         │  <- Dynamic, fills terminal
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│ ● Watching: Dune (2021)  ████████░░░ 1h 43m / 2h 35m          │  <- Player status (1 row)
├─────────────────────────────────────────────────────────────────┤
│ j↓ k↑  /Search  f Fav  d DL  Enter Select  ? Help  q Quit     │  <- Context hints (1 row)
└─────────────────────────────────────────────────────────────────┘
```

Total overhead: 5 rows for chrome. Content gets the rest.

### 3.2 Movies Tab Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ [SEARCH BAR] _______________________________  🔍               │
├───────────────────────┬─────────────────────────────────────────┤
│ Results (40%)         │ Details Panel (60%)                     │
│ ─────────────────     │ ─────────────────────────────────────── │
│ > Dune (2021)   ★8.0 │  ┌──────┐  DUNE                        │
│   Oppenheimer   ★8.9 │  │poster│  2021 · Sci-Fi · 2h 35m      │
│   Interstellar  ★8.6 │  │      │  ★ 8.0 / 10  (12,847 votes)  │
│   The Batman    ★7.9 │  └──────┘                               │
│   Avatar 2      ★7.6 │  A noble family becomes embroiled in    │
│   Top Gun 2     ★7.9 │  a war for control of the galaxy's      │
│                       │  most valuable asset...                 │
│   [HD]  [4K]  [Sub]  │                                         │
│                       │  Cast: Timothée Chalamet · Zendaya      │
│                       │  ─────────────────────────────────────  │
│                       │  [▶ Stream 4K]  [▶ Stream HD]  [⬇ DL] │
└───────────────────────┴─────────────────────────────────────────┘
```

### 3.3 Anime Tab Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ [SEARCH BAR] _______________________________  [Sub] [Dub]  🔍  │
├──────────────────────┬──────────────────────────────────────────┤
│ Search Results (35%) │ Episode List / Details (65%)            │
│ ────────────────     │ ──────────────────────────────────────── │
│ > Attack on Titan    │  ATTACK ON TITAN — Season 4              │
│   Demon Slayer       │  ⛩ AniList: Completed  ★ 9.0           │
│   Jujutsu Kaisen     │  ─────────────────────────────────────── │
│   One Piece          │  E01  To You, in 2000 Years              │
│   Naruto Shippuden   │  E02  That Day                           │
│                      │  E03  ▶ Night of the End                 ← Resume
│  [AniList ✓ Synced]  │  E04  From His Perspective              │
│                      │  E05  Declaration of War                 │
│                      │  ─────────────────────────────────────── │
│                      │  [▶ Play E03]  [⬇ Download]  [a Sync]  │
└──────────────────────┴──────────────────────────────────────────┘
```

### 3.4 Sports Tab Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Live Sports                                  ● 14 LIVE NOW      │
├──────────────┬─────────────────────┬─────────────────────────────┤
│ Sports (20%) │ Matches (45%)       │ Streams (35%)               │
│ ──────────── │ ─────────────────── │ ──────────────────────────  │
│ > Football   │ ● Man U 0-1 Arsenal │ > English HD (Sky Sports)   │
│   Cricket    │ ● Chelsea vs City   │   Spanish HD                │
│   Basketball │ ⏰ Liverpool (2h)   │   Arabic HD                 │
│   F1         │ ● Real vs Barça     │   SD Backup                 │
│   Tennis     │                     │                             │
│   Boxing     │  COMPETITION        │  720p ✓  Live  ██████░░    │
│   Darts      │  Premier League     │                             │
│   Baseball   │  Round 32           │  [▶ Play]  [Copy URL]       │
└──────────────┴─────────────────────┴──────────────────────────────┘
```

### 3.5 F1 Tab Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ 🏎 Formula 1 — 2026 Season                                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ ⏰ NEXT: 🏁 JAPANESE GRAND PRIX — Race                   │   │
│  │    Suzuka International Racing Course · Japan            │   │
│  │    ┌─────────────────────────────────────────────────┐   │   │
│  │    │         02d  14h  23m  07s                      │   │   │
│  │    │         ████████████████████░░░░░░░░░░░         │   │   │
│  │    └─────────────────────────────────────────────────┘   │   │
│  │    [▶ Watch Live]  [📋 Sessions]                         │   │
│  └──────────────────────────────────────────────────────────┘   │
├────────┬─────────────────────────┬────────────┬──────────────────┤
│ Round  │ Event                   │ Date       │ Status           │
│ ────── │ ─────────────────────── │ ────────── │ ──────────────── │
│  R01   │ Bahrain Grand Prix      │ Mar 16-18  │ ✓ Complete      │
│  R02   │ Saudi Arabian GP        │ Mar 22-24  │ ✓ Complete      │
│  R03   │ Australian GP           │ Apr 13-15  │ ✓ Complete      │
│ ▶ R04  │ Japanese Grand Prix     │ Apr 27-29  │ 🏁 NEXT RACE   │
│  R05   │ Chinese Grand Prix      │ May 04-06  │ Scheduled       │
└────────┴─────────────────────────┴────────────┴──────────────────┘
│ Enter: Sessions  w: Watch Live  r: Refresh  ? Help             │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. Loading States

Every async operation shows visual feedback:

### 4.1 Search Loading

```
[SEARCH BAR] naruto_________________  🔍

         ⠙ Searching 3 providers...
```

### 4.2 Stream Resolution

```
  ┌───────────────────────────────────┐
  │                                   │
  │   ⠹ Resolving stream...           │
  │   Trying HiAnime (1/2)            │
  │                                   │
  └───────────────────────────────────┘
```

### 4.3 Spinner Frames

```rust
const SPINNER: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
// Advance one frame per Tick action (16ms)
```

---

## 5. Error States

Errors are shown inline, never as popups that block the UI:

```
  ┌──────────────────────────────────┐
  │                                  │
  │   ⚠ HiAnime is down.            │
  │   Trying AllAnime (2/2)...       │
  │                                  │
  │   Press r to retry, Esc to back  │
  │                                  │
  └──────────────────────────────────┘
```

For unrecoverable errors:

```
  ┌──────────────────────────────────┐
  │                                  │
  │   ✗ No providers available.      │
  │                                  │
  │   All sources tried and failed.  │
  │   Check your internet connection │
  │   or try again later.            │
  │                                  │
  │   [r] Retry  [?] Help  [Esc] Back│
  │                                  │
  └──────────────────────────────────┘
```

---

## 6. Poster Rendering

Posters are rendered inline when the terminal supports image protocols.

### Detection Order

```rust
fn detect_image_protocol() -> ImageProtocol {
    // Check TERM_PROGRAM and various env vars
    if env::var("TERM").as_deref() == Ok("xterm-kitty")
       || env::var("TERM_PROGRAM").as_deref() == Ok("ghostty") {
        return ImageProtocol::Kitty;
    }
    if terminal_supports_sixel() {
        return ImageProtocol::Sixel;
    }
    ImageProtocol::BlockUnicode  // fallback: always works
}
```

### Unicode Block Fallback

When no image protocol is available, posters are rendered as block art using `▀`, `▄`, and `█` with 256-color approximation. This ensures posters always display, even in basic terminals.

---

## 7. Settings Screen Design

```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙ Settings                                                     │
├─────────────────┬───────────────────────────────────────────────┤
│ Navigation      │ Player                                        │
│ ──────────────  │ ──────────────────────────────────────────── │
│ > Player        │  Preferred Player:  [mpv        ▼]           │
│   Appearance    │  Custom Command:    _____________________     │
│   Accounts      │  Extra mpv Args:    _____________________     │
│   Providers     │                                               │
│   Quality       │  Detected Players: ✓ mpv  ✓ vlc  ✗ iina    │
│   Downloads     │                                               │
│   Notifications │  Resume Position:  [On  ▼]                  │
│   Keybindings   │  Save History:     [On  ▼]                  │
│   About         │                                               │
│                 │  [Test Player]                                │
└─────────────────┴───────────────────────────────────────────────┘
│ j/k Navigate  Enter Edit  Esc Back  s Save                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Help Screen

```
┌─────────────────────────────────────────────────────────────────┐
│ Sloth v0.1.0 — Keyboard Reference                              │
├───────────────────────┬─────────────────────────────────────────┤
│ Global                │ Movies & TV                             │
│ ─────────────────     │ ──────────────────────────────────────  │
│ / or s   Search       │ Enter    Select / Details               │
│ Tab      Switch tab   │ q        Quality picker                 │
│ 1-8      Jump to tab  │ d        Download                       │
│ ?        This help    │ f        Favorite toggle                │
│ q        Quit         │ Esc      Back                           │
│ t        Next theme   │                                         │
│ p        Player cycle │ Anime                                   │
│ Ctrl+C   Force quit   │ ──────────────────────────────────────  │
│                       │ t        Sub/Dub toggle                 │
│ Navigation            │ a        AniList sync                   │
│ ─────────────────     │ c        Season calendar                │
│ j / ↓    Down         │                                         │
│ k / ↑    Up           │ Sports                                  │
│ h / ←    Left/prev    │ ──────────────────────────────────────  │
│ l / →    Right/next   │ h/l      Switch columns                 │
│ gg       Go to top    │ Enter    Play stream                    │
│ G        Go to bottom │ r        Refresh live data              │
│ PgDn     Page down    │                                         │
│ PgUp     Page up      │ F1                                      │
│ Enter    Confirm       │ ──────────────────────────────────────  │
│ Esc      Cancel/Back  │ Enter    View sessions                  │
│                       │ w        Watch live                     │
│                       │ r        Refresh calendar               │
└───────────────────────┴─────────────────────────────────────────┘
│ Press any key to close                                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. Responsive Design

The layout must adapt to terminal size:

| Width | Behavior |
|---|---|
| ≥ 120 cols | Full two-column layout (poster + details) |
| 80-119 cols | Reduced details panel, no poster |
| 60-79 cols | List only, details on Enter |
| < 60 cols | Minimal: list only, no borders |

| Height | Behavior |
|---|---|
| ≥ 35 rows | Full layout with status bar |
| 25-34 rows | Compact mode, shorter lists |
| < 25 rows | Warning: terminal too small |

```rust
// In render functions:
if area.width < 60 {
    self.render_minimal(area, buf, state, theme);
    return;
}
if area.width < 80 {
    self.render_compact(area, buf, state, theme);
    return;
}
self.render_full(area, buf, state, theme);
```
