# Truthy Rule

## Description
Use this rule to forbid non-explicitly typed truthy values other than allowed ones (by default: `true` and `false`).

## Options
- `allowed-values`: List of allowed truthy values (default: ['true', 'false'])
- `check-keys`: Whether to apply rules to keys in mappings (default: true)

## Default Configuration
```yaml
rules:
  truthy:
    allowed-values: ['true', 'false']
    check-keys: true
```
