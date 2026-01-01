# Theming System

UmbraRelay uses a SCSS-based theming system that allows users to select between different visual themes. Themes are stored in the database and persist across app launches.

## Architecture

### File Structure

```
src/
  styles/
    main.scss              # Main entry point, imports all themes and components
    _variables.scss        # Base SCSS variables (spacing, fonts, etc.)
    _mixins.scss           # SCSS mixins for reusable styles
    themes/
      _light.scss          # Light theme variables (default)
      _dark.scss           # Dark theme variables
      _blue.scss           # Blue theme variables
    components/
      _shared.scss         # Shared component styles
      _app.scss            # App.vue styles
      _inbox-view.scss     # InboxView styles
      _item-detail.scss    # ItemDetail styles
      _source-config.scss  # SourceConfig styles
      ...                  # Other component styles
  composables/
    useTheme.ts            # Theme management composable
```

### Theme System

The theme system works by:

1. **CSS Variables**: Each theme defines CSS custom properties (variables) for colors, backgrounds, borders, etc.
2. **Theme Classes**: Themes are applied by adding classes to the root `<html>` element:
   - `theme-light` - Light theme (also default via `:root`)
   - `theme-dark` - Dark theme
   - `theme-blue` - Blue theme
3. **System Preference**: The "system" theme option detects the OS preference using `prefers-color-scheme` media query and applies light or dark accordingly.

### Theme Variables

Each theme file defines variables for:

- **Primary colors**: Buttons, accents, links
- **Background colors**: Main background, cards, panels, hover states
- **Text colors**: Primary, secondary, muted text
- **Border colors**: Default, hover, light borders
- **State colors**: Success, error, warning, info
- **Badge colors**: Unread, read, archived states
- **Group tag colors**: Group display colors

### Component Styles

Component styles use CSS variables instead of hardcoded colors:

```scss
.my-component {
  background: var(--color-bg-card);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
}
```

This allows themes to automatically apply to all components.

## Usage

### Selecting a Theme

Users can select a theme in the **Sources** configuration panel:

1. Open the Sources view
2. Scroll to the "Theme" section
3. Select one of:
   - **System** - Follows OS preference (default)
   - **Light** - Light theme
   - **Dark** - Dark theme
   - **Blue** - Blue theme

The theme selection is saved to the database and persists across app restarts.

### System Theme Behavior

When "System" is selected:
- The app detects the OS color scheme preference on startup
- The theme automatically updates when the OS preference changes
- The current system preference (Light/Dark) is shown in the theme selector

## Adding a New Theme

To add a new theme:

1. **Create theme file**: `src/styles/themes/_new-theme.scss`
   ```scss
   :root.theme-new-theme {
     --color-primary: #your-color;
     --color-bg-primary: #your-bg;
     // ... define all variables
   }
   ```

2. **Import in main.scss**: Add `@import 'themes/new-theme';`

3. **Update useTheme.ts**: Add the new theme to the `Theme` type:
   ```typescript
   export type Theme = 'system' | 'light' | 'dark' | 'blue' | 'new-theme';
   ```

4. **Add to theme selector**: Update `SourceConfig.vue` to include the new theme option

5. **Update database validation**: Ensure the theme name is included in validation checks

### Theme Variable Checklist

When creating a new theme, define all these variables:

- `--color-primary` and `--color-primary-hover`
- `--color-primary-text`
- `--color-bg-primary`, `--color-bg-secondary`, `--color-bg-card`, `--color-bg-hover`, `--color-bg-panel`, `--color-bg-unread`
- `--color-text-primary`, `--color-text-secondary`, `--color-text-muted`, `--color-text-inverse`
- `--color-border`, `--color-border-hover`, `--color-border-light`
- `--color-success`, `--color-success-bg`
- `--color-error`, `--color-error-bg`
- `--color-warning`, `--color-warning-bg`
- `--color-info`, `--color-info-bg`
- `--color-badge-unread`, `--color-badge-unread-text`
- `--color-badge-read`, `--color-badge-read-text`
- `--color-badge-archived`, `--color-badge-archived-text`
- `--color-group-tag`, `--color-group-tag-text`

## Database Storage

Theme preferences are stored in the `user_preferences` table:

```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

The theme is stored as:
- Key: `theme`
- Value: `system`, `light`, `dark`, or `blue`

## Backend API

The theme system uses two Tauri commands:

- `get_user_preference(key: String)` - Get a preference value
- `set_user_preference(key: String, value: String)` - Set a preference value

These are defined in `src-tauri/src/commands.rs` and use the database methods in `src-tauri/src/storage/db.rs`.

## Best Practices

1. **Always use CSS variables** - Never hardcode colors in component styles
2. **Test all themes** - Ensure your component looks good in light, dark, and blue themes
3. **Use semantic variable names** - Use `--color-bg-card` not `--color-white`
4. **Consider contrast** - Ensure text is readable in all themes
5. **Update all themes** - When adding a new variable, update all theme files

## Troubleshooting

### Theme not applying

- Check that the theme class is applied to `<html>` element
- Verify CSS variables are defined in the theme file
- Check browser console for CSS errors

### System theme not updating

- Verify `prefers-color-scheme` media query is supported
- Check that the media query listener is properly set up
- Ensure `useTheme` composable is initialized in `App.vue`

### Theme not persisting

- Check database connection
- Verify `user_preferences` table exists
- Check Tauri command registration in `lib.rs`

