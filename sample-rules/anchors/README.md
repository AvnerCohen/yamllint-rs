# Anchors Rule

## Description
Use this rule to report duplicated anchors and aliases referencing undeclared anchors.

## Options
- `forbid-undeclared-aliases`: Avoid aliases that reference an anchor that hasn't been declared
- `forbid-duplicated-anchors`: Avoid duplications of a same anchor  
- `forbid-unused-anchors`: Avoid anchors being declared but not used anywhere in the YAML document via alias

## Default Configuration
```yaml
rules:
  anchors:
    forbid-undeclared-aliases: true
    forbid-duplicated-anchors: false
    forbid-unused-anchors: false
```
