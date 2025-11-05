# Braces Rule

## Description
Use this rule to control the use of flow mappings or number of spaces inside braces (`{` and `}`).

## Options
- `forbid`: Forbid the use of flow mappings completely (`true`) or except for empty ones (`non-empty`)
- `min-spaces-inside`: Minimal number of spaces required inside braces
- `max-spaces-inside`: Maximal number of spaces allowed inside braces
- `min-spaces-inside-empty`: Minimal number of spaces required inside empty braces
- `max-spaces-inside-empty`: Maximal number of spaces allowed inside empty braces

## Default Configuration
```yaml
rules:
  braces:
    forbid: false
    min-spaces-inside: 0
    max-spaces-inside: 0
    min-spaces-inside-empty: -1
    max-spaces-inside-empty: -1
```
