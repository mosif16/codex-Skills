# Color Palette Reference

Quick reference for implementing consistent color schemes across platforms.

## Recommended Accent Colors

Choose ONE for your entire app:

### Professional & Corporate
- **Indigo**: #6366F1 - Professional, trustworthy
- **Slate Blue**: #475569 - Corporate, serious
- **Navy**: #1E3A8A - Traditional, corporate

### Energetic & Modern
- **Emerald**: #10B981 - Fresh, growth-oriented
- **Cyan**: #06B6D4 - Tech-forward, modern
- **Violet**: #8B5CF6 - Creative, bold

### Warm & Friendly
- **Amber**: #F59E0B - Optimistic, approachable
- **Orange**: #F97316 - Energetic, enthusiastic
- **Rose**: #F43F5E - Passionate, attention-grabbing

### Specialized
- **Teal**: #14B8A6 - Healthcare, wellness
- **Lime**: #84CC16 - Environmental, sustainable

## Neutral Palette (Use for 80% of UI)

```
// Backgrounds
White: #FFFFFF
Off-white: #FAFAFA
Light gray 1: #F5F5F5
Light gray 2: #EEEEEE

// Borders & dividers
Border gray: #E5E5E5
Divider gray: #D4D4D4

// Text & UI elements
Medium gray: #9E9E9E
Dark gray: #757575
Darker gray: #424242
Near-black: #212121
True black: #000000 (use sparingly)
```

## Color Combinations

### Example 1: Professional App (Indigo accent)
```
Primary action: #6366F1 (Indigo)
Background: #FFFFFF (White)
Surface: #F5F5F5 (Light gray)
Text primary: #212121 (Near-black)
Text secondary: #757575 (Dark gray)
Border: #E5E5E5 (Border gray)
```

### Example 2: Health/Wellness App (Emerald accent)
```
Primary action: #10B981 (Emerald)
Background: #FAFAFA (Off-white)
Surface: #FFFFFF (White)
Text primary: #212121 (Near-black)
Text secondary: #757575 (Dark gray)
Border: #E5E5E5 (Border gray)
```

### Example 3: Financial App (Slate Blue accent)
```
Primary action: #475569 (Slate blue)
Background: #FFFFFF (White)
Surface: #F5F5F5 (Light gray)
Text primary: #1E293B (Dark slate)
Text secondary: #64748B (Medium slate)
Border: #E2E8F0 (Light slate)
```

## Semantic Colors (Use sparingly)

```
Success: #10B981 (Emerald green)
Warning: #F59E0B (Amber)
Error: #EF4444 (Red)
Info: #3B82F6 (Blue)

// Use these only for:
// - Success/error states in forms
// - Status indicators
// - Alert messages
// Never as primary UI colors
```

## Platform-Specific Color Implementation

### SwiftUI
```swift
extension Color {
    static let accentPrimary = Color(hex: "#6366F1")
    static let textPrimary = Color(hex: "#212121")
    static let textSecondary = Color(hex: "#757575")
    static let borderGray = Color(hex: "#E5E5E5")
    static let backgroundLight = Color(hex: "#F5F5F5")
}

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 6: (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        default: (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue:  Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}
```

### Android (Jetpack Compose)
```kotlin
// colors.xml or in Compose
val AccentPrimary = Color(0xFF6366F1)
val TextPrimary = Color(0xFF212121)
val TextSecondary = Color(0xFF757575)
val BorderGray = Color(0xFFE5E5E5)
val BackgroundLight = Color(0xFFF5F5F5)

// In Theme
lightColorScheme(
    primary = AccentPrimary,
    onPrimary = Color.White,
    background = Color.White,
    surface = BackgroundLight,
    onSurface = TextPrimary,
    outline = BorderGray
)
```

### CSS/Web
```css
:root {
    --accent-primary: #6366F1;
    --text-primary: #212121;
    --text-secondary: #757575;
    --border-gray: #E5E5E5;
    --background-light: #F5F5F5;
    --background-white: #FFFFFF;
    
    --success: #10B981;
    --warning: #F59E0B;
    --error: #EF4444;
}

/* Usage */
.button-primary {
    background-color: var(--accent-primary);
    color: var(--background-white);
}
```

## Anti-Patterns

**AVOID:**
- Multiple accent colors (pick ONE)
- Gradients (especially purple/blue)
- Pure black #000000 for text (too harsh)
- Pure saturated colors (hard to look at)
- Different color on every component
- Using semantic colors (red, green) as primary colors

**INSTEAD:**
- One accent color + neutrals
- Solid colors
- Near-black #212121 or dark gray
- Slightly desaturated colors
- Consistent color application
- Reserve semantic colors for their purpose
