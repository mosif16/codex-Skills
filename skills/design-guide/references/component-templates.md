# Component Templates

Ready-to-use code templates for common UI components following design guide principles.

## Button Components

### SwiftUI Primary Button

```swift
struct PrimaryButton: View {
    let title: String
    let action: () -> Void
    @State private var isPressed = false
    
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(.white)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .padding(.horizontal, 24)
                .background(Color.accentPrimary)
                .cornerRadius(8)
                .shadow(color: Color.black.opacity(0.12), radius: 3, x: 0, y: 1)
        }
        .scaleEffect(isPressed ? 0.98 : 1.0)
    }
}

// Usage
PrimaryButton(title: "Continue") {
    // Action
}
```

### Android Compose Primary Button

```kotlin
@Composable
fun PrimaryButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Button(
        onClick = onClick,
        modifier = modifier
            .fillMaxWidth()
            .height(48.dp),
        shape = RoundedCornerShape(8.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = AccentPrimary,
            contentColor = Color.White
        ),
        elevation = ButtonDefaults.buttonElevation(
            defaultElevation = 2.dp,
            pressedElevation = 4.dp
        )
    ) {
        Text(
            text = text,
            fontSize = 16.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
}
```

### React Primary Button

```jsx
const PrimaryButton = ({ children, onClick, disabled = false }) => {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="w-full py-3 px-6 bg-indigo-600 text-white font-semibold 
                 rounded-lg shadow-sm hover:bg-indigo-700 active:bg-indigo-800
                 disabled:opacity-40 disabled:cursor-not-allowed
                 transition-all duration-200 ease-in-out
                 hover:shadow-md active:scale-98"
    >
      {children}
    </button>
  );
};

// CSS if not using Tailwind
const buttonStyles = {
  button: {
    width: '100%',
    padding: '12px 24px',
    backgroundColor: '#6366F1',
    color: 'white',
    fontSize: '16px',
    fontWeight: '600',
    border: 'none',
    borderRadius: '8px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.12)',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  }
};
```

## Card Components

### SwiftUI Card

```swift
struct ContentCard<Content: View>: View {
    let content: Content
    
    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            content
        }
        .padding(24)
        .background(Color.white)
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.1), radius: 6, x: 0, y: 4)
    }
}

// Usage
ContentCard {
    VStack(alignment: .leading, spacing: 8) {
        Text("Title")
            .font(.system(size: 20, weight: .semibold))
            .foregroundColor(Color.textPrimary)
        
        Text("Subtitle")
            .font(.system(size: 14))
            .foregroundColor(Color.textSecondary)
        
        Divider()
            .padding(.vertical, 16)
        
        Text("Card content goes here...")
            .font(.system(size: 16))
            .foregroundColor(Color.textPrimary)
    }
}
```

### Android Compose Card

```kotlin
@Composable
fun ContentCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(
            containerColor = Color.White
        ),
        elevation = CardDefaults.cardElevation(
            defaultElevation = 4.dp
        )
    ) {
        Column(
            modifier = Modifier.padding(24.dp),
            content = content
        )
    }
}

// Usage
ContentCard {
    Text(
        text = "Title",
        fontSize = 20.sp,
        fontWeight = FontWeight.SemiBold,
        color = TextPrimary
    )
    Spacer(modifier = Modifier.height(8.dp))
    Text(
        text = "Subtitle",
        fontSize = 14.sp,
        color = TextSecondary
    )
    Spacer(modifier = Modifier.height(16.dp))
    Divider(color = BorderGray)
    Spacer(modifier = Modifier.height(16.dp))
    Text(
        text = "Card content...",
        fontSize = 16.sp,
        color = TextPrimary
    )
}
```

### React Card

```jsx
const Card = ({ children, className = "" }) => {
  return (
    <div 
      className={`bg-white rounded-xl p-6 shadow-md ${className}`}
    >
      {children}
    </div>
  );
};

// Usage
<Card>
  <h2 className="text-xl font-semibold text-gray-900">Title</h2>
  <p className="text-sm text-gray-600 mt-2">Subtitle</p>
  <hr className="my-4 border-gray-200" />
  <p className="text-base text-gray-900">Card content...</p>
</Card>
```

## Form Input Components

### SwiftUI Text Field

```swift
struct StyledTextField: View {
    let label: String
    let placeholder: String
    @Binding var text: String
    var error: String? = nil
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(label)
                .font(.system(size: 14, weight: .semibold))
                .foregroundColor(Color.textPrimary)
            
            TextField(placeholder, text: $text)
                .padding(.horizontal, 16)
                .padding(.vertical, 12)
                .font(.system(size: 16))
                .background(Color.white)
                .cornerRadius(8)
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(error != nil ? Color.red : Color.borderGray, lineWidth: 1)
                )
            
            if let error = error {
                Text(error)
                    .font(.system(size: 14))
                    .foregroundColor(.red)
            }
        }
    }
}
```

### Android Compose Text Field

```kotlin
@Composable
fun StyledTextField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String = "",
    error: String? = null,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier) {
        Text(
            text = label,
            fontSize = 14.sp,
            fontWeight = FontWeight.SemiBold,
            color = TextPrimary,
            modifier = Modifier.padding(bottom = 8.dp)
        )
        
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            placeholder = { Text(placeholder) },
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(8.dp),
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = AccentPrimary,
                unfocusedBorderColor = BorderGray,
                errorBorderColor = Color.Red
            ),
            isError = error != null,
            singleLine = true
        )
        
        error?.let {
            Text(
                text = it,
                fontSize = 14.sp,
                color = Color.Red,
                modifier = Modifier.padding(top = 4.dp)
            )
        }
    }
}
```

### React Text Input

```jsx
const TextInput = ({ 
  label, 
  placeholder, 
  value, 
  onChange, 
  error 
}) => {
  return (
    <div className="mb-6">
      <label className="block text-sm font-semibold text-gray-900 mb-2">
        {label}
      </label>
      <input
        type="text"
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        className={`
          w-full px-4 py-3 text-base border rounded-lg
          focus:outline-none focus:ring-2 focus:ring-indigo-600
          ${error 
            ? 'border-red-500 focus:ring-red-500' 
            : 'border-gray-300'
          }
        `}
      />
      {error && (
        <p className="mt-1 text-sm text-red-600">{error}</p>
      )}
    </div>
  );
};
```

## Navigation Components

### SwiftUI Bottom Tab Bar

```swift
struct CustomTabBar: View {
    @Binding var selectedTab: Int
    let tabs: [(icon: String, label: String)]
    
    var body: some View {
        HStack(spacing: 0) {
            ForEach(tabs.indices, id: \.self) { index in
                Button(action: { selectedTab = index }) {
                    VStack(spacing: 4) {
                        Image(systemName: tabs[index].icon)
                            .font(.system(size: 24))
                        Text(tabs[index].label)
                            .font(.system(size: 12))
                    }
                    .foregroundColor(selectedTab == index ? Color.accentPrimary : Color.textSecondary)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                }
            }
        }
        .frame(height: 56)
        .background(Color.white)
        .shadow(color: Color.black.opacity(0.1), radius: 6, x: 0, y: -4)
    }
}
```

### Android Compose Bottom Navigation

```kotlin
@Composable
fun BottomNavigationBar(
    items: List<BottomNavItem>,
    selectedItem: Int,
    onItemSelected: (Int) -> Unit
) {
    NavigationBar(
        containerColor = Color.White,
        modifier = Modifier.height(56.dp)
    ) {
        items.forEachIndexed { index, item ->
            NavigationBarItem(
                selected = selectedItem == index,
                onClick = { onItemSelected(index) },
                icon = {
                    Icon(
                        imageVector = item.icon,
                        contentDescription = item.label,
                        modifier = Modifier.size(24.dp)
                    )
                },
                label = {
                    Text(
                        text = item.label,
                        fontSize = 12.sp
                    )
                },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = AccentPrimary,
                    selectedTextColor = AccentPrimary,
                    unselectedIconColor = TextSecondary,
                    unselectedTextColor = TextSecondary
                )
            )
        }
    }
}

data class BottomNavItem(
    val icon: ImageVector,
    val label: String
)
```

### React Bottom Navigation

```jsx
const BottomNav = ({ items, activeIndex, onSelect }) => {
  return (
    <nav className="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 h-14 shadow-lg">
      <div className="flex h-full">
        {items.map((item, index) => (
          <button
            key={index}
            onClick={() => onSelect(index)}
            className={`
              flex-1 flex flex-col items-center justify-center gap-1
              ${activeIndex === index ? 'text-indigo-600' : 'text-gray-500'}
            `}
          >
            <item.icon className="w-6 h-6" />
            <span className="text-xs">{item.label}</span>
          </button>
        ))}
      </div>
    </nav>
  );
};
```

## Spacing Helpers

### SwiftUI
```swift
extension CGFloat {
    static let spacing8: CGFloat = 8
    static let spacing16: CGFloat = 16
    static let spacing24: CGFloat = 24
    static let spacing32: CGFloat = 32
    static let spacing48: CGFloat = 48
    static let spacing64: CGFloat = 64
}

// Usage
.padding(.vertical, .spacing16)
.padding(.horizontal, .spacing24)
```

### Android
```xml
<!-- res/values/dimens.xml -->
<resources>
    <dimen name="spacing_xs">8dp</dimen>
    <dimen name="spacing_sm">16dp</dimen>
    <dimen name="spacing_md">24dp</dimen>
    <dimen name="spacing_lg">32dp</dimen>
    <dimen name="spacing_xl">48dp</dimen>
    <dimen name="spacing_xxl">64dp</dimen>
</resources>
```

### Web/CSS
```css
:root {
    --spacing-xs: 8px;
    --spacing-sm: 16px;
    --spacing-md: 24px;
    --spacing-lg: 32px;
    --spacing-xl: 48px;
    --spacing-xxl: 64px;
}

/* Or Tailwind custom config */
module.exports = {
  theme: {
    spacing: {
      '2': '8px',
      '4': '16px',
      '6': '24px',
      '8': '32px',
      '12': '48px',
      '16': '64px',
    }
  }
}
```

## Key Principles Reminder

When using these templates:
1. Always use 8px grid spacing
2. Minimum touch target 44x44px (iOS) or 48x48px (Android)
3. One accent color throughout
4. Clear interactive states (hover, pressed, disabled)
5. Neutral colors as base (80%), accent sparingly (5%)
6. Body text minimum 16px
7. Subtle shadows only
8. Cards use border OR shadow, not both
