---
layout: api-command
language: Python
permalink: api/python/slice/
command: 'slice, []'
related_commands:
    order_by: order_by/
    skip: skip/
    limit: limit/
    'nth, []': nth/
---

# Command syntax #

{% apibody %}
selection.slice(start_offset[, end_offset, left_bound='closed', right_bound='open']) &rarr; selection
stream.slice(start_offset[, end_offset, left_bound='closed', right_bound='open']) &rarr; stream
array.slice(start_offset[, end_offset, left_bound='closed', right_bound='open']) &rarr; array
binary.slice(start_offset[, end_offset, left_bound='closed', right_bound='open']) &rarr; binary
string.slice(start_offset[, end_offset, left_bound='closed', right_bound='open']) &rarr; string
{% endapibody %}

# Description #

Return the elements of a sequence within the specified range.

`slice` returns the range between `start_offset` and `end_offset`. If only `start_offset` is specified, `slice` returns the range from that index to the end of the sequence. Specify `left_bound` or `right_bound` as `open` or `closed` to indicate whether to include that endpoint of the range by default: `closed` returns that endpoint, while `open` does not. By default, `left_bound` is closed and `right_bound` is open, so the range `(10,13)` will return the tenth, eleventh and twelfth elements in the sequence.

If `end_offset` is past the end of the sequence, all elements from `start_offset` to the end of the sequence will be returned. If `start_offset` is past the end of the sequence or `end_offset` is less than `start_offset`, a zero-element sequence will be returned.

Negative `start_offset` and `end_offset` values are allowed with arrays; in that case, the returned range counts back from the array's end. That is, the range `(-2)` returns the last two elements, and the range of `(2,-1)` returns the second element through the next-to-last element of the range. An error will be raised on a negative `start_offset` or `end_offset` with non-arrays. (An `end_offset` of &minus;1 *is* allowed with a stream if `right_bound` is closed; this behaves as if no `end_offset` was specified.)

If `slice` is used with a [binary](/api/python/binary) object, the indexes refer to byte positions within the object. That is, the range `(10,20)` will refer to the 10th byte through the 19th byte.

With a string, `slice` behaves similarly, with the indexes referring to Unicode codepoints. String indexes start at `0`. (Note that [combining codepoints][cc] are counted separately.)

[cc]: https://en.wikipedia.org/wiki/Combining_character

If you are only specifying the indexes and not the bounding options, you may use Python's slice operator as a shorthand: `[start_offset:end_offset]`.

__Example:__ Return the fourth, fifth and sixth youngest players. (The youngest player is at index 0, so those are elements 3&ndash;5.)

```py
r.table('players').order_by(index='age').slice(3,6).run(conn)
```

Or, using Python's slice operator:

```py
r.table('players').filter({'class': 'amateur'})[10:20].run(conn)
```

__Example:__ Return all but the top three players who have a red flag.

```py
r.table('players').filter({'flag': 'red'}).order_by(index=r.desc('score')).slice(3).run(conn)
```

__Example:__ Return holders of tickets `X` through `Y`, assuming tickets are numbered sequentially. We want to include ticket `Y`.

```py
r.table('users').order_by(index='ticket').slice(x, y, right_bound='closed').run(conn)
```

__Example:__ Return the elements of an array from the second through two from the end (that is, not including the last two).

```py
r.expr([0,1,2,3,4,5]).slice(2,-2).run(conn)
[2,3]
```

__Example:__ Return the third through fifth characters of a string.

```py
> r.expr("rutabaga").slice(2,5).run(conn)
"tab"
```
