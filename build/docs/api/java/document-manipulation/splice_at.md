---
layout: api-command
language: Java
permalink: api/java/splice_at/
command: spliceAt
related_commands:
    insertAt: insert_at/
    deleteAt: delete_at/
    changeAt: change_at/
---

# Command syntax #

{% apibody %}
array.spliceAt(offset, array) &rarr; array
{% endapibody %}

# Description #

Insert several values into an array at the given index. Returns the modified array.

__Example:__ Hulk and Thor decide to join the Avengers.

```java
r.expr(r.array("Iron Man", "Spider-Man"))
 .spliceAt(1, r.array("Hulk", "Thor")).run(conn);
```

