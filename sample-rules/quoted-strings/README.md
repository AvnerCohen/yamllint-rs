# Quoted Strings Rule

## Description
Use this rule to control the use of quoted strings.

## Options
- `quote-type`: Allowed quotes (`single`, `double`, or `any`)
- `required`: Whether quotes are required (`true`, `false`, or `only-when-needed`)
- `extra-required`: List of regexes to force quoting
- `extra-allowed`: List of regexes to allow quoted strings
- `allow-quoted-quotes`: Allow disallowed quotes for strings with allowed quotes inside
- `check-keys`: Apply rules to keys in mappings

## Default Configuration
```yaml
rules:
  quoted-strings:
    quote-type: any
    required: true
    extra-required: []
    extra-allowed: []
    allow-quoted-quotes: false
    check-keys: false
```

## Special Configuration Required
⚠️ **This rule is NOT enabled by default in yamllint.** You must explicitly enable it in your configuration file.

To test this rule, create a config file:
```yaml
rules:
  quoted-strings:
    required: true
```

Then run: `yamllint -c config.yaml file.yaml`
