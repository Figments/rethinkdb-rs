---
layout: documentation
title: Asynchronous connections
docs_active: async-connections
permalink: docs/async-connections/
---

Certain RethinkDB drivers support asynchronous connections by integrating with popular async libraries. This is particularly useful with [changefeeds][cf] and other real-time applications.

[cf]: /docs/changefeeds

Due to its event-driven nature, JavaScript can easily execute RethinkDB queries in an asynchronous fashion. The official RethinkDB drivers currently support integration with EventMachine for Ruby, and Tornado and Twisted for Python.

{% toctag %}

# JavaScript

No special procedures or commands are necessary to execute RethinkDB queries asynchronously in JavaScript. Read about using callbacks and promises with RethinkDB in the documentation for the [run][] command.

[run]: /api/javascript/run/

In addition, RethinkDB's cursors and feeds implement an [EventEmitter interface][ee] compatible with Node's. This allows your application to set up listeners to receive data from queries as the data becomes available.

[ee]: /api/javascript/ee-cursor/

# Ruby with EventMachine

The RethinkDB Ruby driver adds a new ReQL command, [em_run](/api/ruby/em_run), designed to work with [EventMachine](http://rubyeventmachine.com). In addition, it provides a superclass, `RethinkDB::Handler`, with event-specific methods (e.g., `on_open`, `on_close`) that may be overridden by a class your application defines and passes to `em_run`.

## Simple usage

The easiest way to use RethinkDB with EventMachine is simply by passing a block to `em_run`. If RethinkDB returns a sequence (including a stream), the block will be called once with each element of the sequence. Otherwise, the block will be called just once with the returned value.

__Example:__ Iterate over a stream

```rb
require 'eventmachine'
require 'rethinkdb'
include RethinkDB::Shortcuts

conn = r.connect(host: 'localhost', port: 28015)

EventMachine.run {
  r.table('test').order_by(:index => 'id').em_run(conn) { |row|
    # do something with returned row data
    p row
  }
}
```

## Explicitly closing a query

The `em_run` command returns a `QueryHandle` instance. The `QueryHandle` will be closed when all results have been received, or when EventMachine stops running. You can explicitly close it with the `close` method.

```rb
EventMachine.run {
  printed = 0
  handle = r.table('test').order_by(:index => 'id').em_run(conn) { |row|
    printed += 1
    if printed > 3
      handle.close
    else
      p row
    end
  }
}
```

## Handling errors

In the form above&mdash;with a block that accepts a single argument&mdash;RethinkDB's EventMachine adapter will throw errors back up to your application for you to handle in the same fashion as you would using RethinkDB without EventMachine. If the table `test` did not exist in the database above, you would receive the standard `ReqlRunTimeError`:

```
RethinkDB::ReqlRunTimeError: Table `test.test` does not exist.
Backtrace:
r.table('test')
^^^^^^^^^^^^^^^
```

You can also choose to receive errors in the block by accepting two arguments. 

```rb
EventMachine.run {
  r.table('test').order_by(:index => 'id').em_run(conn) { |err, row|
  if err
    p [:err, err.to_s]
  else
    p [:row, row]
  end
  }
}
```

In this form, the block will receive `nil` as the first argument if there is no error. In the case of an error, the second argument will be `nil`.

## Using RethinkDB::Handler

To gain more precise control, write a class that inherits from `RethinkDB::Handler` and override the event handling methods, then pass an instance of that class to `em_run`.

__Example:__ Iterate over a stream using a handler

```rb
require 'eventmachine'
require 'rethinkdb'
include RethinkDB::Shortcuts

conn = r.connect(host: 'localhost', port: 28015)

class Printer < RethinkDB::Handler

  def on_open
    p :open
  end
  
  def on_close
    p :closed
  end
  
  def on_error(err)
    p [:err, err.to_s]
  end
  
  def on_val(val)
    p [:val, val]
  end

end

EventMachine.run {
  r.table('test').order_by(:index => 'id').em_run(conn, Printer)
}

# Sample output
:open
[:val, {"id"=>1}]
[:val, {"id"=>2}]
[:val, {"id"=>3}]
:closed
```

## Distinguishing between data types

In addition to the simple `on_val` method, you can provide methods that specifically apply to arrays, streams and atoms.

```rb
class Printer < RethinkDB::Handler

  def on_open
    p :open
  end
  
  def on_close
    p :closed
  end
  
  def on_error(err)
    p [:err, err.to_s]
  end
  
  # Handle arrays
  def on_array(array)
    p [:array, array]
  end
  
  # Handle atoms
  def on_atom(atom)
    p [:atom, atom]
  end
  
  # Handle individual values received from streams
  def on_stream_val(val)
    p [:stream_val, val]
  end
  
  def on_val(val)
    p [:val, val]
  end

end

EventMachine.run {
  r.table('test').order_by(:index => 'id').em_run(conn, Printer)
  # print an array
  r.expr([1, 2, 3]).em_run(conn, Printer)
  # print a single row
  r.table('test').get(1).em_run(conn, Printer)
}

# Sample output
:open
[:stream_val, {"id"=>0}]
[:stream_val, {"id"=>1}]
[:stream_val, {"id"=>2}]
:closed
:open
[:array, [1, 2, 3]]
:closed
:open
[:atom, {"id"=>0}]
:closed
```

The various `on_*` methods provide fallbacks for one another:

* an array will be handled by `on_array` if defined; otherwise it will be handled by `on_atom`. If neither of those are defined, the *individual elements* of the array will be handled by `on_stream_val` or, if that is not defined, `on_val`.
* a stream will be handled by `on_stream_val` if defined; otherwise it will be handled by `on_val`.
* data that is *not* a stream will be handled by `on_atom` if defined; otherwise it will be handled by `on_val`.

Thus, `on_val` acts a "catch-all" for any data that is not handled by a more specific method.

The order in which callbacks are called in the `EventMachine.run` block is not guaranteed; in the sample output above, `[:array, [1, 2, 3]]` might have printed first.

## Changefeeds

A changefeed is handled like any other stream; when you pass a block to `em_run`, the block is called with each document received on the feed. If you pass a `Handler` that defines `on_stream_val` (or `on_val`), those methods will be called with each document.

In addition, there are changefeed-specific methods that may be defined.

* `on_initial_val`: if the changefeed returns initial values (`include_initial` has been specified as an option to [changes](/api/ruby/changes/), those values will be passed to this method.
* `on_uninitial_val`: a changefeed that returns initial values may also return "uninitial" values to indicate a document already sent as an initial value has been changed (see the `changes` documentation for details); those values, if any, will be passed to this method.
* `on_change`: changes will be passed to this method.
* `on_change_error`: if the feed includes a document specifying errors that do not cause the feed to abort (for instance, a notification the server discarded some changes), those errors will be passed to this method.
* `on_state`: a feed may include documents specifying the state of the stream; those documents will be passed to this function if defined.

```rb
class FeedPrinter < RethinkDB::Handler

  def on_open
    p :open
  end
  
  def on_close
    p :closed
  end
  
  def on_error(err)
    p [:err, err.to_s]
  end
  
  def on_initial_val(val)
    p [:initial, val]
  end
  
  def on_state(state)
    p [:state, state]
  end
  
  def on_change(old, new)
    p [:change, old, new]
  end  

end

# Subscribe to changes on the documents with the two lowest ids
EventMachine.run {
  r.table('test').order_by(:index => 'id').limit(2).changes
    .em_run(conn, FeedPrinter)
}

# Sample output
:open
[:state, "initializing"]
[:initial_val, {"id"=>1}]
[:initial_val, {"id"=>0}]
[:state, "ready"]

# Execute: r.table('test').insert({id: 0.5}).run(conn)
[:change, {"id"=>1}, {"id"=>0.5}]

# Execute: r.table_drop('test').run(conn)
[:err, "Changefeed aborted (table unavailable).\nBacktrace..."]
:closed
```

## Using one Handler with multiple queries

You can register multiple queries with the same `Handler` instance. If you define `Handler` methods with an additional argument (two arguments instead of one, or one argument instead of zero), that argument will receive the appropriate `QueryHandle` instance.

```rb
class MultiQueryPrinter < RethinkDB::Handler
  
  def on_open(qh)
    p [:open, names[qh]]
  end
  
  def on_close(qh)
    p [:close, names[qh]]
    EventMachine.stop if @closed == 2
  end
  
  def on_val(val, qh)
    p [:val, val, names[qh]]
  end
  
end

EventMachine.run {
  printer = Printer.new
  h1 = r.expr(1).em_run(conn, printer)
  h2 = r.expr(2).em_run(conn, printer)
  names = { h1 => "h1", h2 => "h2" }
}

# Sample output
[:open, "h1"]
[:val, 1, "h1"]
[:close, "h1"]
[:open, "h2"]
[:val, 2, "h2"]
[:close, "h2"]
```

## Stopping a Handler

If you call the `stop` method on a `Handler`, it will stop processing changes and open streams using that handler will be closed. Queries registered with that handler instance will not be interrupted if they are currently processing (e.g., a batch write), but will close rather than executing after the handler has been stopped.

__Example:__ Print the first five changes to a table. After the handler has been stopped, the changefeed query will be closed on the next change to the table rather than returning a value.

```rb
class FeedPrinter < RethinkDB::Handler
  
  def initialize(max)
    @counter = max
    stop if @counter <= 0
  end
  
  def on_open
    # Once the changefeed is open, insert 10 rows
    r.table('test').insert([{}] * 10).run(conn, noreply: true)
  end
  
  def on_val(val)
    # Every time we print a change, decrement @counter and stop if we hit 0
    p val
    @counter -= 1
    stop if @counter <= 0
  end
  
end

EventMachine.run {
  r.table('test').changes.em_run(conn, Printer.new(5))
}
```

# Python with Tornado or Twisted

The RethinkDB Python driver integrates with both the [Tornado web framework][tor] and the [Twisted networking engine][twi]. By using the [set_loop_type][slt] command, you can select either the `'tornado'` or `'twisted'` event loop model, returning Tornado `Future` objects or Twisted `Deferred` objects respectively.

[tor]: http://www.tornadoweb.org/
[twi]: http://twistedmatrix.com/
[slt]: /api/python/set_loop_type
[con]: /api/python/connect

## Tornado

### Basic Usage

Before `connect`, use the `set_loop_type("tornado")` command to set RethinkDB to use asynchronous event loops compatible with Tornado.

```py
import rethinkdb as r
from tornado import ioloop, gen
from tornado.concurrent import Future, chain_future
import functools

r.set_loop_type("tornado")
connection = r.connect(host='localhost', port=28015)
```

After executing `set_loop_type`, `r.connect` will return a Tornado `Future`, as will `r.run`.

__Example:__ Simple use

```py
@gen.coroutine
def single_row(connection_future):
    # Wait for the connection to be ready
    connection = yield connection_future
    # Insert some data
    yield r.table('test').insert([{"id": 0}, {"id": 1}, {"id": 2}]).run(connection)
    # Print the first row in the table
    row = yield r.table('test').get(0).run(connection)
    print(row)

# Output
{u'id': 0}
```

__Example:__ Using a cursor

```py
@gen.coroutine
def use_cursor(connection_future):
    # Wait for the connection to be ready
    connection = yield connection_future
    # Insert some data
    yield r.table('test').insert([{"id": 0}, {"id": 1}, {"id": 2}]).run(connection)
    # Print every row in the table.
    cursor = yield r.table('test').order_by(index="id").run(connection)
    while (yield cursor.fetch_next()):
        item = yield cursor.next()
        print(item)

# Output
{u'id': 0}
{u'id': 1}
{u'id': 2}
```

Note that looping over a cursor must be done with `while` and `fetch_next`, rather than using a `for x in cursor` loop.

### Error handling

If an error occurs during an asynchronous operation, the `yield` statement will throw an exception as normal. This may happen immediately (for example, you might reference a table that doesn't exist), but your application might receive large amounts of data before the error (for example, your network might be disrupted after the connection is established).

One error in particular is notable. If you have a coroutine set to consume a changefeed indefinitely, and the connection closes, the coroutine will experience a `ReqlRuntimeError`.

__Example:__ Re-thrown errors

```py
@gen.coroutine
def bad_table(connection):
    yield r.table('non_existent').run(connection)

Traceback (most recent call last):
... elided ...
rethinkdb.errors.ReqlRuntimeError: Table `test.non_existent` does not exist. in:
r.table('non_existent')
^^^^^^^^^^^^^^^^^^^^^^^
```

__Example:__ Catching errors in the coroutine

```py
@gen.coroutine
def catch_bad_table(connection):
    try:
        yield r.table('non_existent').run(connection)
    except r.ReqlRuntimeError:
        print("Saw error")

# Output
Saw error
```

### Subscribing to changefeeds

The asynchronous database API allows you to handle multiple changefeeds simultaneously by scheduling background coroutines. As an example, consider this changefeed handler:

```py
@gen.coroutine
def print_cfeed_data(connection_future, table):
    connection = yield connection_future
    feed = yield r.table(table).changes().run(connection)
    while (yield feed.fetch_next()):
        item = yield feed.next()
        print(item)
```

We can schedule it on the Tornado IO loop with this code:

```py
ioloop.IOLoop.current().add_callback(print_cfeed_data, connection, table)
```

Now the coroutine will run in the background, printing out changes. When we alter the table, the changes will be noticed.

Now, consider a larger example.

```py
class ChangefeedNoticer(object):
    def __init__(self, connection):
        self._connection = connection
        self._sentinel = object()
        self._cancel_future = Future()
    @gen.coroutine
    def print_cfeed_data(self, table):
        feed = yield r.table(table).changes().run(self._connection)
        self._feeds_ready[table].set_result(True)
        while (yield feed.fetch_next()):
            cursor = feed.next()
            chain_future(self._cancel_future, cursor)
            item = yield cursor
            if item is self._sentinel:
                return
            print("Seen on table %s: %s" % (table, item))
    @gen.coroutine
    def table_write(self, table):
        for i in range(10):
            yield r.table(table).insert({'id': i}).run(self._connection)
    @gen.coroutine
    def exercise_changefeeds(self):
        self._feeds_ready = {'a': Future(), 'b': Future()}
        loop = ioloop.IOLoop.current()
        loop.add_callback(self.print_cfeed_data, 'a')
        loop.add_callback(self.print_cfeed_data, 'b')
        yield self._feeds_ready
        yield [self.table_write('a'), self.table_write('b')]
        self._cancel_future.set_result(self._sentinel)
    @classmethod
    @gen.coroutine
    def run(cls, connection_future):
        connection = yield connection_future
        if 'a' in (yield r.table_list().run(connection)):
            yield r.table_drop('a').run(connection)
        yield r.table_create('a').run(connection)
        if 'b' in (yield r.table_list().run(connection)):
            yield r.table_drop('b').run(connection)
        yield r.table_create('b').run(connection)
        noticer = cls(connection)
        yield noticer.exercise_changefeeds()

# Output
Seen on table a: {u'old_val': None, u'new_val': {u'id': 0}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 0}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 1}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 1}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 2}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 2}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 3}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 3}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 4}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 4}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 5}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 6}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 5}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 7}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 6}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 8}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 7}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 9}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 8}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 9}}
```

Here, we listen for changes on multiple tables at once.  We simultaneously write into the tables, and observe our writes appear in the changefeeds.  We then cancel the changefeeds after we've written 10 items into each of the tables.

## Twisted

### Basic Usage

Before `connect`, use the `set_loop_type("twisted")` command to set RethinkDB to use asynchronous event loops compatible with the Twisted reactor.
 
```py
import rethinkdb as r
from twisted.internet import reactor, defer
from twisted.internet.defer import inlineCallbacks, returnValue

r.set_loop_type('twisted')
connection = r.connect(host='localhost', port=28015)
```

After executing `set_loop_type`, `r.connect` will return a Twisted `Deferred`, as will `r.run`.

__Example:__ Simple use

```py
@inlineCallbacks
def single_row(conn_deferred):
    # Wait for the connection to be ready
    conn = yield conn_deferred
    # Insert some data
    yield r.table('test').insert([{"id": 0}, {"id": 1}, {"id": 2}]).run(conn)
    # Print the first row in the table
    row = yield r.table('test').get(0).run(conn)
    print(row)

# Output
{u'id': 0}
```

__Example:__ Using a cursor

```py
@inlineCallbacks
def use_cursor(conn):
    # Insert some data
    yield r.table('test').insert([{"id": 0}, {"id": 1}, {"id": 2}]).run(conn)
    # Print every row in the table.
    cursor = yield r.table('test').order_by(index="id").run(conn)
    while (yield cursor.fetch_next()):
        item = yield cursor.next()
        print(item)

# Output:
{u'id': 0}
{u'id': 1}
{u'id': 2}
```

Note that looping over a cursor must be done with `while` and `fetch_next`, rather than using a `for x in cursor` loop.

### Error handling

If an error occurs during an asynchronous operation, the `yield` statement will throw an exception as normal. This may happen immediately (for example, you might reference a table that doesn’t exist), but your application might receive large amounts of data before the error (for example, your network might be disrupted after the connection is established).

One error in particular is notable. If you have a task that consumes a changefeed indefinitely, and the connection closes, the task will experience a `ReqlRuntimeError`.

__Example:__ Re-thrown errors

```py
@inlineCallbacks
def bad_table(conn):
    yield r.table('non_existent').run(conn)

Unhandled error in Deferred:
Traceback (most recent call last):
Failure: rethinkdb.errors.ReqlOpFailedError: Table `test.non_existent` does not exist in:
r.table('non_existent')
^^^^^^^^^^^^^^^^^^^^^^^

```

__Example:__ Catching runtime errors

```py
@inlineCallbacks
def catch_bad_table(conn):
    try:
        yield r.table('non_existent').run(conn)
    except r.ReqlRuntimeError:
        print("Saw error")

# Output
Saw error
```

### Subscribing to changefeeds

The asynchronous database API allows you to handle multiple changefeeds simultaneously by running multiple background tasks. As an example, consider this changefeed handler:

```py
@inlineCallbacks
def print_feed(conn, table):
    feed = yield r.table(table).changes().run(conn)
    while (yield feed.fetch_next()):
        item = yield feed.next()
        print("Seen on table %s: %s" % (table, str(item)))
```

We can schedule it on the Twisted `reactor` with this code:

```py
reactor.callLater(0, print_cfeed_data, conn, table)
```

Now the task will run in the background, printing out changes. When we alter the table, the changes will be noticed.

Now consider a larger example:

```py
@inlineCallbacks
def print_feed(conn, table, ready, cancel):
    def errback_feed(feed, err):
        feed.close()
        return err

    feed = yield r.table(table).changes().run(conn)
    cancel.addErrback(lambda err: errback_feed(feed, err))
    ready.callback(None)
    while (yield feed.fetch_next()):
        item = yield feed.next()
        print("Seen on table %s: %s" % (table, str(item)))

@inlineCallbacks
def table_write(conn, table):
    for i in range(10):
        yield r.table(table).insert({'id': i}).run(conn)

@inlineCallbacks
def notice_changes(conn, *tables):
    # Reset the state of the tables on the server
    if len(tables) > 0:
        table_list = yield r.table_list().run(conn)
        yield defer.DeferredList([r.table_drop(t).run(conn) for t in tables if t in table_list])
    yield defer.DeferredList([r.table_create(t).run(conn) for t in tables])

    readies = [defer.Deferred() for t in tables]
    cancel = defer.Deferred()
    feeds = [print_feed(conn, table, ready, cancel) for table, ready in zip(tables, readies)]

    # Wait for the feeds to become ready
    yield defer.gatherResults(readies)
    yield defer.gatherResults([table_write(conn, table) for table in tables])

    # Cancel the feeds and wait for them to exit
    cancel.addErrback(lambda err: None)
    cancel.cancel()
    yield defer.DeferredList(feeds)

yield notice_changes(conn, 'a', 'b')

# Output
Seen on table b: {u'old_val': None, u'new_val': {u'id': 0}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 0}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 1}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 1}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 2}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 2}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 3}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 3}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 4}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 4}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 5}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 5}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 6}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 6}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 7}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 7}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 8}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 8}}
Seen on table a: {u'old_val': None, u'new_val': {u'id': 9}}
Seen on table b: {u'old_val': None, u'new_val': {u'id': 9}}
```

Here, we listen for changes on multiple tables at once.  We simultaneously write into the tables, and observe our writes appear in the changefeeds.  We then cancel the changefeeds after we've written 10 items into each of the tables.
