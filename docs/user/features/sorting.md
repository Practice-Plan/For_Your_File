# Sorting Options

Sorting allows you to customize the order in which search results and entries are displayed.

## Overview

![Sorting Options](../images/features/sorting-options.png)

The sorting feature provides:
- **Multiple Sort Methods**: Choose from various sorting strategies
- **Custom Sorting**: Configure weighted sorting
- **Visual Indicators**: See sort order in results
- **Persistent Settings**: Remember your preferences

## Sort Methods

### Available Sort Methods

| Method | Description | Best For |
|--------|-------------|----------|
| **Relevance** | Best match to search query | Search results |
| **Most Used** | Frequently opened entries | Finding favorites |
| **Recently Opened** | Last opened time | Recent work |
| **Alphabetical** | Sort by name A-Z | Browsing all entries |
| **Custom** | Weighted combination | Personalized ranking |

### Relevance Sorting

Default for search results. Factors:
- Search term match quality
- Match position (beginning is better)
- Exact match bonus
- Multiple term matches

![Relevance Sort](../images/features/sort-relevance.png)

### Most Used Sorting

Sort by frequency of access. Factors:
- Open count (frequency)
- Weighted by time decay
- Recent use weighted higher

![Most Used Sort](../images/features/sort-most-used.png)

### Recently Opened Sorting

Sort by last opened timestamp. Shows:
- Entries opened today first
- Then this week
- Then this month
- Then older

![Recently Opened Sort](../images/features/sort-recently-opened.png)

### Alphabetical Sorting

Standard A-Z sorting. Options:
- Ascending (A-Z)
- Descending (Z-A)
- Case-sensitive or insensitive

![Alphabetical Sort](../images/features/sort-alphabetical.png)

### Custom Sorting

Configure your own weighted sorting:

![Custom Sort](../images/features/sort-custom.png)

Combine multiple factors with custom weights.

## Custom Sorting Configuration

### Understanding Weights

Custom sorting uses weighted scores:

```
Total Score = (Frequency × 0.3) + (Recency × 0.2) + (Relevance × 0.5)
```

Each weight must sum to 1.0.

### Configuring Weights

Access custom sorting:

1. Settings > Sorting
2. Select "Custom" method
3. Adjust weight sliders:

![Custom Sort Weights](../images/features/sort-custom-weights.png)

#### Weight Adjustments

| Weight | Range | Effect |
|--------|-------|--------|
| **Frequency** | 0.0 - 1.0 | How much open count matters |
| **Recency** | 0.0 - 1.0 | How much last opened time matters |
| **Relevance** | 0.0 - 1.0 | How much search match matters |

### Weight Presets

Use presets for common scenarios:

| Preset | Frequency | Recency | Relevance | Best For |
|--------|-----------|---------|-----------|----------|
| **Balanced** | 0.33 | 0.33 | 0.34 | General use |
| **Frequent** | 0.7 | 0.2 | 0.1 | Finding favorites |
| **Recent** | 0.2 | 0.7 | 0.1 | Current work |
| **Search** | 0.1 | 0.2 | 0.7 | Search-focused |
| **Custom** | User-defined | | | Personal preference |

### Frequency Half-Life

Configure the frequency decay:

![Frequency Half-Life](../images/features/sort-half-life.png)

| Half-Life | Effect |
|-----------|--------|
| 7 days | Recent use strongly preferred |
| 30 days | Balanced between recent and old |
| 90 days | Long-term frequency matters more |
| Never | Pure frequency count |

## Time Window Grouping

Group results by time windows:

![Time Window Groups](../images/features/sort-time-windows.png)

### Time Window Categories

| Window | Description |
|--------|-------------|
| **Hour** | Last 60 minutes |
| **Today** | Since midnight today |
| **Week** | Last 7 days |
| **Month** | Last 30 days |
| **Older** | More than 30 days |

Results are grouped and sorted within each group.

## Sort Options in UI

### Sort Dropdown

Quickly change sort method:

![Sort Dropdown](../images/features/sort-dropdown.png)

Location: Top of results panel

### Sort Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + Shift + R` | Sort by Relevance |
| `Ctrl + Shift + F` | Sort by Most Used |
| `Ctrl + Shift + T` | Sort by Recently Opened |
| `Ctrl + Shift + A` | Sort Alphabetically |
| `Ctrl + Shift + C` | Sort Custom |

### Right-Click Sorting

Right-click results header:

![Right-Click Sort](../images/features/sort-right-click.png)

Options:
- Change sort method
- Reverse sort order
- Group by time window
- Show sort indicators

## Visual Indicators

### Sort Direction Indicator

![Sort Indicator](../images/features/sort-indicator.png)

- ▲ Ascending (A-Z)
- ▼ Descending (Z-A)

### Score Breakdown (Debug Mode)

Enable debug mode to see score breakdown:

![Score Breakdown](../images/features/sort-debug.png)

Shows:
- Frequency score
- Recency score
- Relevance score
- Total score

Enable: Settings > Sorting > Debug Mode

## Sorting in Different Views

### Search Results

Search results default to Relevance sorting:

- Best matches first
- Customizable in Settings > Search
- Can override per search

### Group View

Group view defaults to Most Used:

- Frequently used items first
- Consistent with usage patterns
- Can change to any sort method

### All Entries View

All entries default to Alphabetical:

- Easy browsing
- Predictable order
- Can change to any sort method

### Expired Entries View

Expired entries sort by expiration date:

- Soonest to expire first
- Or most recently expired
- Helps prioritize cleanup

## Advanced Sorting

### Multi-Level Sorting

Sort by multiple criteria:

![Multi-Level Sort](../images/features/sort-multi-level.png)

Example:
1. Primary: Group
2. Secondary: Most Used
3. Tertiary: Alphabetical

### Sort Profiles

Save and load sort profiles:

![Sort Profiles](../images/features/sort-profiles.png)

Create profiles for:
- Different workflows
- Different projects
- Different views

### Sort Scripting

Advanced users can create custom sort scripts:

```javascript
function customSort(a, b) {
  // Custom sorting logic
  return a.score - b.score;
}
```

## Performance Considerations

### Index-Based Sorting

All sorting uses pre-built indexes:
- No performance impact on search
- Fast sorting of large result sets
- Indexes updated on changes

### Sorting Large Datasets

For 10,000+ entries:
- Custom sorting may be slightly slower
- Consider using simpler sort methods
- Use time windows to limit results

## Troubleshooting

### Sorting Not Applied

- Check if sort method is saved
- Verify no filters override sort
- Try refreshing the view
- Check for conflicting settings

### Custom Sort Not Working

- Ensure weights sum to 1.0
- Verify entries have required data
- Check for null values
- Try resetting to defaults

### Results Order Unexpected

- Check sort direction (ascending/descending)
- Verify sort method is correct
- Check for group-based sorting
- Review time window settings

---

See also:
- [Search](./search.md) - Search functionality
- [Groups](./groups.md) - Organizing entries
- [Keyboard Shortcuts](../keyboard-shortcuts.md) - Sort shortcuts

*Last updated: 2026*