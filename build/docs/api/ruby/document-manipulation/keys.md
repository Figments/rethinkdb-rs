---
layout: api-command
language: Ruby
permalink: api/ruby/keys/
command: keys
related_commands:
    values: values/
---

# Command syntax #

{% apibody %}
singleSelection.keys() &rarr; array
object.keys() &rarr; array
{% endapibody %}

# Description #

Return an array containing all of an object's keys. Note that the keys will be sorted as described in [ReQL data types](/docs/data-types/#sorting-order) (for strings, lexicographically).

__Example:__ Get all the keys from a table row.

```rb
# row: { :id => 1, :mail => "fred@example.com", :name => "fred" }

r.table('users').get(1).keys().run(conn)

> [ "id", "mail", "name" ]
```
