# code-weather

Code quality metrics, but make it meteorology.

## Why This Exists

Every code quality tool gives you the same thing: numbers. Cyclomatic complexity 47. Test coverage 63%. Lines changed 1,247.

Cool. What does that mean?

code-weather translates metrics into weather forecasts. "Stormy with poor visibility" tells you more than "CC: 47, CRAP: 892" ever could. It's the difference between "barometric pressure dropping" and "bring an umbrella."

## Features

- **Weather Forecasts**: Sunny, Partly Cloudy, Cloudy, Rainy, Stormy, Foggy, Frozen
- **Temperature**: Maps git activity to degrees (hot = active, frozen = abandoned)
- **Humidity**: Test coverage as moisture (dry = no tests, humid = well covered)
- **Wind**: Churn velocity and direction (gale force = unstable)
- **Visibility**: Documentation coverage (fog = undocumented)
- **Multi-language**: TypeScript, JavaScript, Python, Rust, Go
- **Multiple Outputs**: Terminal (with ASCII art), JSON, Markdown

## Quick Start

```bash
# Install
cargo install code-weather

# Get the forecast
code-weather forecast ./your-project

# JSON for CI pipelines
code-weather forecast ./src --format json

# Markdown for docs
code-weather forecast --format markdown > WEATHER.md
```

## Usage

### Forecast

```bash
# Basic forecast
code-weather forecast

# Specific path
code-weather forecast ./src

# Different output formats
code-weather forecast --format terminal   # ASCII art (default)
code-weather forecast --format json       # Machine readable
code-weather forecast --format markdown   # For docs

# Skip git analysis (faster, less accurate)
code-weather forecast --no-git

# Verbose mode
code-weather -v forecast
```

### Init

```bash
# Create config file
code-weather init

# Overwrite existing
code-weather init --force
```

### Explain

```bash
# List all conditions
code-weather explain

# Explain specific condition
code-weather explain stormy
```

## Weather Conditions

| Condition | Icon | What It Means |
|-----------|------|---------------|
| Sunny | sun | Clean code, good tests, active maintenance |
| Partly Cloudy | cloud-sun | Good shape with minor issues |
| Cloudy | cloud | Moderate concerns, some tech debt |
| Rainy | rain | Significant issues need attention |
| Stormy | storm | Critical problems, complexity through the roof |
| Foggy | fog | Code works but nobody knows how |
| Frozen | snow | Abandoned, no recent activity |

## Configuration

### Config Precedence

Settings are loaded in this order (highest priority first):

1. **CLI flags** - Always win
2. **Environment variables** - `CODE_WEATHER_*`
3. **Project config** - `.code-weather.toml` in project root
4. **User config** - `~/.config/code-weather/config.toml`
5. **Built-in defaults**

### Project Config

Create `.code-weather.toml` in your project root:

```toml
[thresholds]
sunny_coverage = 80    # Test coverage for sunny weather
cloudy_coverage = 50   # Below this gets cloudy

[analysis]
exclude = ["node_modules", "vendor", "target", ".git"]
```

### User Config

For settings that apply to all projects, create `~/.config/code-weather/config.toml`:

```toml
[display]
temp_unit = "fahrenheit"
color = true

[thresholds]
sunny_coverage = 85
```

### Environment Variables

Override any setting via environment variables:

```bash
# Thresholds
CODE_WEATHER_SUNNY_COVERAGE=90
CODE_WEATHER_CLOUDY_COVERAGE=60
CODE_WEATHER_SUNNY_COMPLEXITY=8
CODE_WEATHER_CLOUDY_COMPLEXITY=15

# Analysis
CODE_WEATHER_SKIP_TESTS=true
CODE_WEATHER_SKIP_GIT=true
CODE_WEATHER_GIT_DEPTH=50

# Display
CODE_WEATHER_NO_COLOR=true
CODE_WEATHER_TEMP_UNIT=celsius
```

Useful for CI pipelines where you want consistent thresholds across repos.

## Sample Output

Terminal:
```
CODE WEATHER FORECAST
=====================

Condition: Partly Cloudy
Temperature: 72F (comfortable activity)
Humidity: 65% (good test coverage)
Wind: 12 mph NW (moderate churn, improving)
Visibility: Good

Overall: Looking decent. A few clouds on the horizon but nothing
to cancel your picnic over.
```

JSON:
```json
{
  "path": "./src",
  "condition": "PartlyCloudy",
  "temperature": {
    "fahrenheit": 72,
    "category": "comfortable"
  },
  "humidity": {
    "percent": 65,
    "category": "comfortable"
  }
}
```

## Supported Languages

| Language | Extensions | Parser |
|----------|------------|--------|
| TypeScript | .ts, .tsx, .mts | tree-sitter-typescript |
| JavaScript | .js, .jsx, .mjs | tree-sitter-javascript |
| Python | .py, .pyi | tree-sitter-python |
| Rust | .rs | tree-sitter-rust |
| Go | .go | tree-sitter-go |

## Performance

- Small repos (< 1k files): < 1 second
- Medium repos (1k-10k files): < 3 seconds
- Large repos (10k-50k files): < 5 seconds

Uses parallel processing via rayon. Respects .gitignore.

## Philosophy

1. Metrics should communicate, not just measure
2. Weather metaphors carry intuitive weight
3. Zero config by default, configurable when needed
4. Fast enough to run on every commit

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Path not found / no source files |
| 2 | Parse error / git error |
| 3 | Config error |
| 4 | IO error |

## License

MIT

## Author

Katie

---

*"Is it sunny or stormy in your codebase?" is a better standup question than "what's our complexity score?"*
