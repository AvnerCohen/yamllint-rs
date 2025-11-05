# Float Values Rule

Limits the permitted values for floating-point numbers.

## Options

- `require-numeral-before-decimal`: Require floats to start with a numeral (e.g., `0.0` instead of `.0`)
- `forbid-scientific-notation`: Forbid scientific notation
- `forbid-nan`: Forbid NaN (not a number) values
- `forbid-inf`: Forbid infinite values

## Default Configuration

```yaml
rules:
  float-values:
    forbid-inf: false
    forbid-nan: false
    forbid-scientific-notation: false
    require-numeral-before-decimal: false
```

## Special Configuration

This rule is disabled by default and needs to be explicitly enabled.
