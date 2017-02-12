---
layout: api-command
language: Ruby
permalink: api/ruby/group/
command: group
related_commands:
    ungroup: ungroup/
    map: map/
    reduce: reduce/
    count: count/
    sum: sum/
    avg: avg/
    min: min/
    max: max/
---

# Command syntax #

{% apibody %}
sequence.group(field | function..., [:index => <indexname>, :multi => true]) &rarr; grouped_stream
r.group(sequence, field | function..., [:index => <indexname>, :multi => true]) &rarr; grouped_stream
{% endapibody %}

<img src="/assets/images/docs/api_illustrations/group.png" class="api_command_illustration" />

# Description #

Takes a stream and partitions it into multiple groups based on the
fields or functions provided.

With the `multi` flag single documents can be assigned to multiple groups, similar to the behavior of [multi-indexes](/docs/secondary-indexes/ruby). When `multi` is `true` and the grouping value is an array, documents will be placed in each group that corresponds to the elements of the array. If the array is empty the row will be ignored.

Suppose that the table `games` has the following data:

```rb
[
    {"id" => 2, "player" => "Bob", "points" => 15, "type" => "ranked"},
    {"id" => 5, "player" => "Alice", "points" => 7, "type" => "free"},
    {"id" => 11, "player" => "Bob", "points" => 10, "type" => "free"},
    {"id" => 12, "player" => "Alice", "points" => 2, "type" => "free"}
]
```

__Example:__ Group games by player.

```rb
> r.table('games').group('player').run(conn)

{
    "Alice" => [
        {"id" => 5, "player" => "Alice", "points" => 7, "type" => "free"},
        {"id" => 12, "player" => "Alice", "points" => 2, "type" => "free"}
    ],
    "Bob" => [
        {"id" => 2, "player" => "Bob", "points" => 15, "type" => "ranked"},
        {"id" => 11, "player" => "Bob", "points" => 10, "type" => "free"}
    ]
}
```

<!-- stop -->

Commands chained after `group` will be called on each of these grouped
sub-streams, producing grouped data.

__Example:__ What is each player's best game?

```rb
> r.table('games').group('player').max('points').run(conn)

{
    "Alice" => {"id" => 5, "player" => "Alice", "points" => 7, "type" => "free"},
    "Bob" => {"id" => 2, "player" => "Bob", "points" => 15, "type" => "ranked"}
}
```

Commands chained onto grouped data will operate on each grouped datum,
producing more grouped data.

__Example:__ What is the maximum number of points scored by each player?

```rb
> r.table('games').group('player').max('points')['points'].run(conn)

{
    "Alice" => 7,
    "Bob" => 15
}
```

You can also group by more than one field.

__Example:__ What is the maximum number of points scored by each
player for each game type?

```rb
> r.table('games').group('player', 'type').max('points')['points'].run(conn)

{
    ["Alice", "free"] => 7,
    ["Bob", "free"] => 10,
    ["Bob", "ranked"] => 15,
    ...
}
```


You can also group by a function.

__Example:__ What is the maximum number of points scored by each
player for each game type?

```rb
> r.table('games')
    .group{|game| game.pluck('player', 'type')}
    .max('points')['points'].run(conn)

{
    {"player" => "Alice", "type" => "free"} => 7,
    {"player" => "Bob", "type" => "free"} => 10,
    {"player" => "Bob", "type" => "ranked"} => 15
}
```

Using a function, you can also group by date on a ReQL [date field](/docs/dates-and-times/javascript/).

__Example:__ How many matches have been played this year by month?

```rb
> r.table('matches').group { |match|
      [match['date'].year(), match['date'].month()]
  }.count().run(conn)

{
    [2014, 2] => 2,
    [2014, 3] => 2,
    [2014, 4] => 1,
    [2014, 5] => 3
}
```

You can also group on an index (primary key or secondary).

__Example:__ What is the maximum number of points scored by game type?


```rb
> r.table('games').group(:index => 'type').max('points')['points'].run(conn)

{
    "free" => 10,
    "ranked" => 15
}
```

# Organizing by value with **multi** #

Suppose that the table `games2` has the following data:

```rb
[
    { :id => 1, :matches => {:a => [1, 2, 3], :b => [4, 5, 6]} },
    { :id => 2, :matches => {:b => [100], :c => [7, 8, 9]} },
    { :id => 3, :matches => {:a => [10, 20], :c => [70, 80]} }
]
```

Using the `multi` option we can group data by match A, B or C.

```rb
> r.table('games2').group(:multi => true){ |row| row['matches'].keys() }.run

[
    {
        :group => 'a',
        :reduction => [ <id 1>, <id 3> ]
    },
    {
        :group => 'b',
        :reduction => [ <id 1>, <id 2> ]
    },
    {
        :group => 'c',
        :reduction => [ <id 2>, <id 3> ]
    }
]
```

(The full result set is abbreviated in the figure; `<id 1>, <id 2>` and `<id 3>` would be the entire documents matching those keys.)

__Example:__ Use [map](/api/ruby/map) and [sum](/api/ruby/sum) to get the total points scored for each match.

```rb
r.table('games3').group(:multi => true){ |row| row['matches'].keys()
}.ungroup().map{ |doc|
    {
        :match => doc['group'], :total => doc['reduction'].sum{ |set|
            set['matches'][doc['group']].sum()
        }
    }
}.run(conn)

[
    { 'match': 'a', 'total': 36 },
    { 'match': 'b', 'total': 115 },
    { 'match': 'c', 'total': 174 }
]
```

The inner `sum` adds the scores by match within each document; the outer `sum` adds those results together for a total across all the documents.

# Ungrouping #

If you want to operate on all the groups rather than operating on each
group (e.g. if you want to order the groups by their reduction), you
can use [ungroup](/api/ruby/ungroup/) to turn a grouped stream or
grouped data into an array of objects representing the groups.

__Example:__ Ungrouping grouped data.

```rb
> r.table('games').group('player').max('points')['points'].ungroup().run(conn)

[
    {
        "group" => "Alice",
        "reduction" => 7
    },
    {
        "group" => "Bob",
        "reduction" => 15
    }
]
```

Ungrouping is useful e.g. for ordering grouped data, or for inserting
grouped data into a table.

__Example:__ What is the maximum number of points scored by each
player, with the highest scorers first?

```rb
> r.table('games')
   .group('player').max('points')['points']
   .ungroup().order_by(r.desc('reduction')).run(conn)

[
    {
        "group" => "Bob",
        "reduction" => 15
    },
    {
        "group" => "Alice",
        "reduction" => 7
    }
]
```

# Implementation Details #

When grouped data are returned to the client, they are transformed
into a client-specific native type.  (Something similar is done with
[times](/docs/dates-and-times/).)  In Ruby, grouped data are
transformed into a `Hash`.  If you instead want to receive the raw
pseudotype from the server (e.g. if you're planning to serialize the
result as JSON), you can specify `group_format: 'raw'` as an optional
argument to `run`:

__Example:__ Get back the raw `GROUPED_DATA` pseudotype.


```rb
> r.table('games').group('player').avg('points').run(conn, group_format:'raw')

{
    "$reql_type$" => "GROUPED_DATA",
    "data" => [
        ["Alice", 4.5],
        ["Bob", 12.5]
    ]
}
```

Not passing the `group_format` flag would return:

```rb
{
    "Alice" => 4.5,
    "Bob" => 12.5
}
```


You might also want to use the [ungroup](/api/ruby/ungroup/)
command (see above), which will turn the grouped data into an array of
objects on the server.


# Performance Details #

If you run a query that returns a grouped stream, it will be
automatically converted to grouped data before being sent back to you
(there is currently no efficient way to stream groups from RethinkDB).
This grouped data is subject to the array size limit, by default 100,000 elements (see [run](/api/ruby/run) for details on how to use the `array_limit` option to change this).

In general, operations on grouped streams will be efficiently
distributed, and operations on grouped data won't be.  You can figure
out what you're working with by putting `type_of` on the end of your
query.  Below are efficient and inefficient examples.

__Example:__ Efficient operation.

```rb
# r.table('games').group('player').type_of().run(conn)
# Returns "GROUPED_STREAM"
r.table('games').group('player').min('points').run(conn) # EFFICIENT
```

__Example:__ Inefficient operation.

```rb
# r.table('games').group('player').order_by('score').type_of().run(conn)
# Returns "GROUPED_DATA"
r.table('games').group('player').order_by('score').nth(0).run(conn) # INEFFICIENT
```

What does it mean to be inefficient here?  When operating on grouped
data rather than a grouped stream, *all* of the data has to be
available on the node processing the query.  This means that the
operation will only use one server's resources, and will require
memory proportional to the size of the grouped data it's operating
on.  (In the case of the [order_by](/api/ruby/order_by/) in the inefficient example, that
means memory proportional **to the size of the table**.)  The array
limit is also enforced for grouped data, so the `order_by` example
would fail for tables with more than 100,000 rows without changing the `array_limit` option to `run`.

# More Examples #

__Example:__ What is the maximum number of points scored by each
player in free games?

```rb
> r.table('games').filter{|game| game['type'].eq('free')}
   .group('player').max('points')['points'].run(conn)

{
    "Alice" => 7,
    "Bob" => 10
}
```

__Example:__ What is each player's highest even and odd score?

```rb
> r.table('games')
   .group('name', lambda {|game| game['points'] % 2})
   .max('points')['points'].run(conn)

{
    ["Alice", 1] => 7,
    ["Bob", 0] => 10,
    ["Bob", 1] => 15
}
```
