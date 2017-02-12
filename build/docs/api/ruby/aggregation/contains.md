---
layout: api-command
language: Ruby
permalink: api/ruby/contains/
command: contains
related_commands:
    map: map/
    concat_map: concat_map/
    group: group/
---

# Command syntax #

{% apibody %}
sequence.contains([value | predicate_function, ...]) &rarr; bool
r.contains(sequence, [value | predicate_function, ...]) &rarr; bool
{% endapibody %}

# Description #

When called with values, returns `true` if a sequence contains all the
specified values.  When called with predicate functions, returns `true`
if for each predicate there exists at least one element of the stream
where that predicate returns `true`.

Values and predicates may be mixed freely in the argument list.

__Example:__ Has Iron Man ever fought Superman?

```rb
r.table('marvel').get('ironman')[:opponents].contains('superman').run(conn)
```


__Example:__ Has Iron Man ever defeated Superman in battle?

```rb
r.table('marvel').get('ironman')[:battles].contains{|battle|
    battle[:winner].eq('ironman') & battle[:loser].eq('superman')
}.run(conn)
```

__Example:__ Return all heroes who have fought _both_ Loki and the Hulk.

```rb
r.table('marvel').filter{ |hero|
    hero[:opponents].contains('loki', 'hulk')
}.run(conn)
```

__Example:__ Use `contains` with a predicate function to simulate an `or`. Return the Marvel superheroes who live in Detroit, Chicago or Hoboken.

```rb
r.table('marvel').filter { |hero|
    r.expr(['Detroit', 'Chicago', 'Hoboken']).contains(hero['city'])
}.run(conn)
```
