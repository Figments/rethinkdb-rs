---
layout: api-command
language: Python
permalink: api/python/order_by/
alias:
    - api/python/asc/
    - api/python/desc/
command: order_by
related_commands:
    skip: skip/
    limit: limit/
    '[]': slice/
---

# Command syntax #

{% apibody %}
table.order_by([key | function], index=index_name) &rarr; table_slice
selection.order_by(key | function[, ...]) &rarr; selection<array>
sequence.order_by(key | function[, ...]) &rarr; array
{% endapibody %}

# Description #

Sort the sequence by document values of the given key(s). To specify
the ordering, wrap the attribute with either `r.asc` or `r.desc`
(defaults to ascending).

__Note:__ RethinkDB uses byte-wise ordering for `orderBy` and does not support Unicode collations; non-ASCII characters will be sorted by UTF-8 codepoint. For more information on RethinkDB's sorting order, read the section in [ReQL data types](/docs/data-types/#sorting-order).

Sorting without an index requires the server to hold the sequence in
memory, and is limited to 100,000 documents (or the setting of the `arrayLimit` option for [run](/api/python/run)). Sorting with an index can
be done on arbitrarily large tables, or after a [between](/api/python/between/) command
using the same index. This applies to both secondary indexes and the primary key (e.g., `index='id'`).

Sorting functions passed to `order_by` must be deterministic. You cannot, for instance, order rows using the [random](/api/python/random/) command. Using a non-deterministic function with `order_by` will raise a `ReqlQueryLogicError`.

__Example:__ Order all the posts using the index `date`.   

```py
r.table('posts').order_by(index='date').run(conn)
```

<!-- stop -->

The index must either be the primary key or have been previously created with [index_create](/api/python/index_create/).

```py
r.table('posts').index_create('date').run(conn)
```

You can also select a descending ordering:

```py
r.table('posts').order_by(index=r.desc('date')).run(conn, callback)
```

__Example:__ Order a sequence without an index.

```py
r.table('posts').get(1)['comments'].order_by('date')
```

You can also select a descending ordering:

```py
r.table('posts').get(1)['comments'].order_by(r.desc('date'))
```

If you're doing ad-hoc analysis and know your table won't have more then 100,000
elements (or you've changed the setting of the `arrayLimit` option for [run](/api/python/run)) you can run `order_by` without an index:

```py
r.table('small_table').order_by('date')
```

__Example:__ You can efficiently order using multiple fields by using a
[compound index](http://www.rethinkdb.com/docs/secondary-indexes/python/).

Order by date and title.

```py
r.table('posts').order_by(index='date_and_title').run(conn)
```

The index must either be the primary key or have been previously created with [index_create](/api/python/index_create/).

```py
r.table('posts').index_create('date_and_title', lambda post:
    [post["date"], post["title"]]).run(conn)
```

_Note_: You cannot specify multiple orders in a compound index. See [issue #2306](https://github.com/rethinkdb/rethinkdb/issues/2306)
to track progress.

__Example:__ If you have a sequence with fewer documents than the `array_limit`, you can order it
by multiple fields without an index.

```py
r.table('small_table').order_by('date', r.desc('title'))
```

__Example:__ Notice that an index ordering always has highest
precedence. The following query orders posts by date, and if multiple
posts were published on the same date, they will be ordered by title.

```py
r.table('post').order_by('title', index='date').run(conn)
```
__Example:__ You can use [nested field](/docs/cookbook/python/#filtering-based-on-nested-fields) syntax to sort on fields from subdocuments. (You can also create indexes on nested fields using this syntax with `index_create`.)

```py
r.table('user').order_by(lambda user: user['group']['id']).run(conn)
```

__Example:__ You can efficiently order data on arbitrary expressions using indexes.

```py
r.table('posts').order_by(index='votes').run(conn)
```

The index must have been previously created with [index_create](/api/ruby/index_create/).

```py
r.table('posts').index_create('votes', lambda post:
    post["upvotes"]-post["downvotes"]
).run(conn)
```

__Example:__ If you have a sequence with fewer documents than the `array_limit`, you can order it with an arbitrary function directly.

```py
r.table('small_table').order_by(lambda doc:
    doc['upvotes']-doc['downvotes']
);
```

You can also select a descending ordering:

```py
r.table('small_table').order_by(r.desc(lambda doc:
    doc['upvotes']-doc['downvotes']
));
```

__Example:__ Ordering after a `between` command can be done as long as the same index is being used.

```py
r.table("posts").between(r.time(2013, 1, 1, '+00:00'), r.time(2013, 1, 1, '+00:00'), index='date')
    .order_by(index='date').run(conn);
```


