# Brackets Rule

## Description
Use this rule to control the use of flow sequences or the number of spaces inside brackets (`[` and `]`).

## Options
- `forbid`: Forbid the use of flow sequences completely (`true`) or except for empty ones (`non-empty`)
- `min-spaces-inside`: Minimal number of spaces required inside brackets
- `max-spaces-inside`: Maximal number of spaces allowed inside brackets
- `min-spaces-inside-empty`: Minimal number of spaces required inside empty brackets
- `max-spaces-inside-empty`: Maximal number of spaces allowed inside empty brackets

## Default Configuration
```yaml
rules:
  brackets:
    forbid: false
    min-spaces-inside: 0
    max-spaces-inside: 0
    min-spaces-inside-empty: -1
    max-spaces-inside-empty: -1
```
