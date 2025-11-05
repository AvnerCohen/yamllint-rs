# YAML Lint Rules - Sample Examples

This directory contains comprehensive examples of YAML linting rules based on the [yamllint documentation](https://yamllint.readthedocs.io/en/stable/rules.html).

## Structure

Each rule has its own directory containing:
- `README.md` - Rule description and configuration options
- `good.yaml` - Examples of YAML that PASS the rule
- `bad.yaml` - Examples of YAML that FAIL the rule

## Available Rules

### Core Formatting Rules
- **anchors** - Control anchor and alias usage (enabled by default)
- **braces** - Control flow mapping formatting (enabled by default)
- **brackets** - Control flow sequence formatting (enabled by default)
- **colons** - Control spacing around colons (enabled by default)
- **commas** - Control spacing around commas (enabled by default)
- **hyphens** - Control spacing after hyphens (enabled by default)
- **indentation** - Enforce consistent indentation (enabled by default)
- **line-length** - Set maximum line length (enabled by default)
- **trailing-spaces** - Forbid trailing spaces (enabled by default)
- **empty-lines** - Control empty line usage (enabled by default)
- **new-lines** - Control newline character type (enabled by default)
- **new-line-at-end-of-file** - Require newline at file end (enabled by default)

### Content Rules
- **key-duplicates** - Forbid duplicate keys (enabled by default)
- **key-ordering** - Force alphabetical key ordering (disabled by default)
- **truthy** - Control truthy value usage (enabled by default)
- **octal-values** - Forbid octal values (disabled by default)
- **quoted-strings** - Control string quoting (disabled by default)
- **comments** - Control comment formatting (enabled by default)
- **comments-indentation** - Control comment indentation (enabled by default)
- **empty-values** - Forbid empty values (disabled by default)
- **float-values** - Control floating-point number formats (disabled by default)

### Document Structure Rules
- **document-start** - Require document start marker `---` (enabled by default)
- **document-end** - Require document end marker `...` (disabled by default)

## Usage

These examples serve as:
1. **Test cases** for implementing the linter
2. **Documentation** for rule behavior
3. **Validation** for rule correctness
4. **Reference** for developers

## Rule Categories

### Formatting Rules
Rules that control the visual appearance and structure of YAML:
- Indentation, line length, spacing
- Flow vs block style formatting
- Empty lines and trailing spaces

### Content Rules  
Rules that control the semantic content of YAML:
- Key uniqueness and ordering
- Value types and formats
- Comment placement and style

### YAML-Specific Rules
Rules that deal with YAML language features:
- Anchors and aliases
- Truthy values and octal numbers
- String quoting requirements

## Implementation Notes

Each rule can be:
- **Enabled/disabled** globally
- **Configured** with specific options
- **Set to different levels** (error, warning, info)
- **Applied selectively** to different parts of the document

The examples show both the **default behavior** and **configurable options** for each rule.
