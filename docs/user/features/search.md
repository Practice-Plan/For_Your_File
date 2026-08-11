# Search Functionality

The search feature is the heart of LNK File Management Center, providing instant access to all your shortcuts.

## Overview

![Search Interface](../images/features/search-interface.png)

The search functionality offers:
- **Real-time search**: Results appear as you type
- **Fuzzy matching**: Find shortcuts even with typos
- **Advanced filters**: Narrow down results by type, group, or tags
- **Search history**: Quick access to recent searches

## Using Search

### Basic Search

1. Press `Alt + Space` to open the search window
2. Type your search query in the search box
3. Results appear instantly below the search box
4. Use arrow keys to navigate results
5. Press `Enter` to open the selected shortcut

![Basic Search](../images/features/search-basic.png)

### Search Query Examples

| Query | Results |
|-------|---------|
| `chrome` | All shortcuts containing "chrome" |
| `work project` | Shortcuts with both "work" and "project" |
| `*.lnk` | All LNK files (wildcard) |
| `tag:important` | Shortcuts tagged as "important" |
| `group:work` | Shortcuts in the "work" group |

### Advanced Search Syntax

Use special operators for precise searches:

| Operator | Example | Description |
|----------|---------|-------------|
| `tag:` | `tag:urgent` | Search by tag |
| `group:` | `group:personal` | Search by group |
| `type:` | `type:url` | Search by type (file/folder/url) |
| `path:` | `path:C:\Users` | Search by target path |
| `created:` | `created:today` | Search by creation date |
| `opened:` | `opened:week` | Search by last opened time |

### Date Search Keywords

Use natural language for date-based searches:

- `today`, `yesterday`
- `week`, `this week`, `last week`
- `month`, `this month`, `last month`
- Specific dates: `2024-01-15`

### Search Filters

Click the filter icon to access advanced filters:

![Search Filters](../images/features/search-filters.png)

#### Filter Options

| Filter | Options |
|--------|---------|
| **Type** | All, Files, Folders, URLs |
| **Group** | Select specific group |
| **Status** | Active, Expired, Expiring Soon |
| **Date Range** | Custom date range |
| **Tags** | Select specific tags |

### Search Results

#### Result Display

Each search result shows:

![Search Result](../images/features/search-result-item.png)

1. **Icon**: Shortcut icon or default icon
2. **Name**: Shortcut name
3. **Target Path**: Where the shortcut points to
4. **Tags**: Associated tags (if any)
5. **Frequency Indicator**: How often used
6. **Expiration Status**: If applicable

#### Result Sorting

Results are sorted by relevance by default. Change sorting in settings:

![Sort Options](../images/features/search-sort-options.png)

| Sort Method | Description |
|-------------|-------------|
| **Relevance** | Best match to search query |
| **Most Used** | Frequently opened shortcuts |
| **Recently Opened** | Last opened time |
| **Alphabetical** | Sort by name A-Z |
| **Custom** | Weighted combination |

### Search History

Access your recent searches:

![Search History](../images/features/search-history.png)

- Click in the search box to see recent searches
- Click a recent search to re-execute it
- Recent searches are stored locally

### Keyboard Navigation in Search

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate results |
| `Enter` | Open selected result |
| `Tab` | Move focus to filters |
| `Escape` | Clear search or close window |
| `Ctrl + K` | Focus search box |

## Search Settings

Configure search behavior in Settings > Search:

![Search Settings](../images/features/search-settings.png)

### Available Settings

| Setting | Description | Default |
|---------|-------------|---------|
| **Max Results** | Maximum results to display | 50 |
| **Search Delay** | Delay before searching (ms) | 150 |
| **Highlight Matches** | Highlight matching text | Yes |
| **Show Path** | Display target path in results | Yes |
| **Fuzzy Search** | Enable fuzzy matching | Yes |
| **Case Sensitive** | Case-sensitive search | No |

### Indexing

The application maintains a search index for fast results:

![Search Indexing](../images/features/search-indexing.png)

- **Automatic Indexing**: Index updates automatically when entries are added/modified
- **Manual Reindex**: Settings > Advanced > Rebuild Search Index
- **Index Location**: `%APPDATA%\LNK Management Center\index.db`

## Tips for Effective Search

1. **Use Tags**: Add tags to shortcuts for easier searching
2. **Be Specific**: More specific queries yield better results
3. **Use Operators**: Leverage `tag:` and `group:` for targeted searches
4. **Check Filters**: Use filters to narrow down large result sets
5. **Learn Shortcuts**: Use keyboard navigation for faster access

## Troubleshooting Search Issues

### No Results Found

- Check your search query for typos
- Verify the entry exists in the database
- Try removing filters
- Check if fuzzy search is enabled

### Slow Search

- Reduce the max results setting
- Disable fuzzy matching for faster results
- Rebuild the search index
- Check system performance

### Missing Entries in Results

- Verify the entry is not expired
- Check if it's in a hidden group
- Rebuild the search index
- Check database integrity

---

See also:
- [Groups](./groups.md) - Organize shortcuts into groups
- [Keyboard Shortcuts](../keyboard-shortcuts.md) - Quick navigation
- [Troubleshooting](../troubleshooting.md) - Common issues

*Last updated: 2026*