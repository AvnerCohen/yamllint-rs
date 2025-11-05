# Key Duplicates Rule

## Description
Use this rule to forbid duplicate keys in mappings. The rule is context-aware and correctly handles complex YAML structures including lists of mappings.

## Key Features
- **Context-aware detection**: Duplicate keys are only flagged within the same mapping context
- **List structure support**: Keys with the same name in different list items are allowed
- **Nested structure support**: Correctly handles deeply nested YAML structures

## Examples

### ✅ Good - No duplicate keys
```yaml
mapping:
  key1: value1
  key2: value2
  key3: value3
```

### ✅ Good - Same keys in different list items (allowed)
```yaml
workflow_steps:
  - name: step1
    type: action
  - name: step2
    type: condition
```

### ❌ Bad - Duplicate keys in same mapping
```yaml
mapping:
  key1: value1
  key2: value2
  key1: value3  # Duplicate key
```

### ❌ Bad - Duplicate keys within same list item
```yaml
workflow_steps:
  - name: step1
    name: duplicate_step  # Duplicate key
    type: action
```

## Options
- `level`: Error level (error or warning)

## Default Configuration
```yaml
rules:
  key-duplicates:
    level: error
```
