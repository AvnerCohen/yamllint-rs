# Octal Values Rule

## Description
Use this rule to forbid octal values in YAML.

## Options
- `forbid-implicit-octal`: Prevent numbers starting with `0` (default: true)
- `forbid-explicit-octal`: Prevent numbers starting with `0o` (default: true)

## Default Configuration
```yaml
rules:
  octal-values:
    forbid-implicit-octal: true
    forbid-explicit-octal: true
```

## Special Configuration Required
⚠️ **This rule is NOT enabled by default in yamllint.** You must explicitly enable it in your configuration file.

To test this rule, create a config file:
```yaml
rules:
  octal-values:
    forbid-implicit-octal: true
    forbid-explicit-octal: true
```

Then run: `yamllint -c config.yaml file.yaml`
