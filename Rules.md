# YAML Lint Rules Reference

This document provides a comprehensive overview of all 23 yamllint rules with examples of good and bad YAML.

## Rule Examples

### Anchors
**Default:** Enabled  
**Description:** Controls YAML anchors and aliases usage (reports duplicated anchors, undeclared aliases, and unused anchors)

| Bad Example | Good Example |
|-------------|--------------|
| <pre>defaults: &defaults<br>  adapter: postgres<br>  host: localhost<br><br>development:<br>  <<: *defaults<br>  database: myapp_development</pre> | <pre>defaults:<br>  adapter: postgres<br>  host: localhost<br><br>development:<br>  adapter: postgres<br>  host: localhost<br>  database: myapp_development</pre> |

---

### Braces
**Default:** Enabled  
**Description:** Controls spacing inside braces `{}`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>map: { key1: value1, key2: value2 }<br>list: [ item1, item2 ]</pre> | <pre>map: {key1: value1, key2: value2}<br>list: [item1, item2]</pre> |

---

### Brackets
**Default:** Enabled  
**Description:** Controls spacing inside brackets `[]`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>list: [ item1, item2, item3 ]<br>nested: [ [1, 2], [3, 4] ]</pre> | <pre>list: [item1, item2, item3]<br>nested: [[1, 2], [3, 4]]</pre> |

---

### Colons
**Default:** Enabled  
**Description:** Controls spacing before and after colons `:`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key : value<br>object  :<br>  - item1<br>  - item2 | <pre>key: value<br>object:<br>  - item1<br>  - item2 |

---

### Commas
**Default:** Enabled  
**Description:** Controls spacing before and after commas `,`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>list: [a, b , c]<br>map: {key1: value1,   key2: value2} | <pre>list: [a, b, c]<br>map: {key1: value1, key2: value2} |

---

### Comments
**Default:** Enabled  
**Description:** Controls comment formatting and spacing

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value#comment<br>#   Bad indentation<br>list:<br>  - item | <pre>key: value  # comment<br># Good indentation<br>list:<br>  - item |

---

### Comments Indentation
**Default:** Enabled  
**Description:** Forces comments to be indented like content

| Bad Example | Good Example |
|-------------|--------------|
| <pre>list:<br>  - item1<br>  - item2<br>#  - item3<br>  - item4 | <pre>list:<br>  - item1<br>  - item2<br>  # - item3<br>  - item4 |

---

### Document End
**Default:** Disabled  
**Description:** Requires document end marker `...`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value | <pre>key: value<br>... |

---

### Document Start
**Default:** Enabled  
**Description:** Requires document start marker `---`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value | <pre>---<br>key: value |

---

### Empty Lines
**Default:** Enabled  
**Description:** Controls empty lines in files

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key1: value1<br><br><br>key2: value2 | <pre>key1: value1<br><br>key2: value2 |

---

### Empty Values
**Default:** Disabled  
**Description:** Forbids empty values in mappings

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key1:<br>key2: ""<br>key3: null | <pre>key1: "not empty"<br>key2: "value"<br>key3: 42 |

---

### Float Values
**Default:** Disabled  
**Description:** Limits permitted floating-point number values

| Bad Example | Good Example |
|-------------|--------------|
| <pre>nan_value: .NaN<br>inf_value: .inf<br>neg_inf: -.inf | <pre>normal_float: 3.14<br>integer: 42<br>string: "hello" |

---

### Hyphens
**Default:** Enabled  
**Description:** Controls spacing after hyphens `-`

| Bad Example | Good Example |
|-------------|--------------|
| <pre>-  item1<br>-  item2<br>-  item3 | <pre>- item1<br>- item2<br>- item3 |

---

### Indentation
**Default:** Enabled  
**Description:** Controls indentation consistency

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key:<br>  subkey:<br>    value<br>  another: test | <pre>key:<br>  subkey:<br>    value<br>  another: test |

---

### Key Duplicates
**Default:** Enabled  
**Description:** Forbids duplicate keys in mappings

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value1<br>key: value2<br>other: test | <pre>key: value<br>other: test<br>another: data |

---

### Key Ordering
**Default:** Disabled  
**Description:** Forces keys to be in alphabetical order

| Bad Example | Good Example |
|-------------|--------------|
| <pre>cherry: red<br>apple: red<br>banana: yellow | <pre>apple: red<br>banana: yellow<br>cherry: red |

---

### Line Length
**Default:** Enabled  
**Description:** Controls maximum line length

| Bad Example | Good Example |
|-------------|--------------|
| <pre>very_long_key_that_exceeds_maximum_line_length: very_long_value_that_also_exceeds_maximum_line_length | <pre>short_key: short_value<br>another: data |

---

### New Line At End Of File
**Default:** Enabled  
**Description:** Requires newline at end of file

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value | <pre>key: value<br> |

---

### New Lines
**Default:** Enabled  
**Description:** Forces type of new line characters

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value | <pre>key: value<br> |

---

### Octal Values
**Default:** Disabled  
**Description:** Forbids octal values in YAML

| Bad Example | Good Example |
|-------------|--------------|
| <pre>permissions: 0755<br>mode: 0644 | <pre>permissions: "0755"<br>mode: "0644" |

---

### Quoted Strings
**Default:** Disabled  
**Description:** Controls when strings must be quoted

| Bad Example | Good Example |
|-------------|--------------|
| <pre>unquoted: value<br>number: 123<br>boolean: true | <pre>quoted: "value"<br>number: "123"<br>boolean: "true" |

---

### Trailing Spaces
**Default:** Enabled  
**Description:** Forbids trailing spaces at end of lines

| Bad Example | Good Example |
|-------------|--------------|
| <pre>key: value   <br>another: test  | <pre>key: value<br>another: test |

---

### Truthy
**Default:** Disabled  
**Description:** Forbids truthy values that can be confusing

| Bad Example | Good Example |
|-------------|--------------|
| <pre>enabled: yes<br>disabled: no<br>active: on | <pre>enabled: "yes"<br>disabled: "no"<br>active: "on" |

---

## Usage

Each rule can be configured in a `.yamllint` configuration file:

```yaml
rules:
  line-length:
    max: 80
  comments:
    min-spaces-from-content: 2
  truthy:
    allowed-values: ['true', 'false']
```

For more detailed information about each rule, see the individual README files in the `sample-rules/` directory.
