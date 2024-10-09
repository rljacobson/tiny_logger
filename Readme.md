# tiny_logger: A Simplistic Tiny Logging Library for Rust

A tiny (~160 LoC) logging library with configurable channels, verbosity, output streams. Developed via accretion for 
my own needs, it is best for when you want to see messages logged to the console but wish to configure the level of 
granularity of those messages dynamically. 

# Logging

There are two orthogonal concepts in the logging infrastructure: a `Channel`, which describes
what _kind_ of messages are to be logged, and a verbosity level (verbosity, level, or log
level), which describes the _verbosity_ of the logger.

Here is a simple example.

```rust
use tiny_logger::{Channel, log};

fn main() {
    // Set the verbosity level to 1. Any messages logged with greater verbosity level
    // will not be emitted.
    tiny_logger::set_verbosity(1);

    // Log messages with different channels and levels
    log(Channel::Critical, 3, "A critical error occurred!");          // Not emitted
    log(Channel::Error,    2, "This is an error message.");           // Not emitted
    log(Channel::Warning,  2, "Warning: Check your input.");          // Not emitted
    log(Channel::Notice,   0, "Notice: This is informational.");      // Emitted
    log(Channel::Info,     1, "Info: Processing started.");           // Emitted
    log(Channel::Debug,    0, "Debug: Variable values are correct."); // Emitted
    log(Channel::Trace,    0, "Trace: Step through the logic here."); // Emitted
}
```

## Verbosity / Logging Levels

Verbosity is a numerical value, with higher values meaning more verbose logging. A verbosity level is a global and
describes how chatty logging is. This global value is the same for all channels. Individual log entries are logged "at"
a given level and are only emitted to the channel if their level is _at most_ the global verbosity level. In other
words, only messages logged at a level less than or equal to the global level are emitted. 

The idea is that you log finer grained details at a higher level. Then you set the global verbosity level higher if you 
want to see those finer grained details and lower if you want a more coarse-grained view.

Note that this is not a filter on the _view_ but rather a filter on what is logged at all. 

## Channels

The `Channel` enum has variants:  `Critical`, `Error`, `Warning`, `Notice`, `Info`, `Debug`, `Trace`. Messages emitted to a channel
are prefixed with the (color coded) channel name.

# Configuring

## Global Verbosity Level

You can set the verbosity level as follows:

```rust
// Set the verbosity to 3.
tiny_logger::set_verbosity(3);
// Messages logged at levels greater than 3 will not be emitted until the verbosity is set to another value.
// ...
// (Re)set the verbosity to 5.
tiny_logger::set_verbosity(5);
// Messages logged at any nonnegative level will now be emitted from here on.
```

## Colors

The colors for each channel are global and are given reasonable defaults. To change these defaults, you can call
`set_color` on a `Channel` variant:

```rust
use yansi::Color;
use tiny_logger::Channel;

Channel::Notice.set_color(Color::Blue);
```

You can also get the color of a channel:

```rust
use yansi::Color;
use tiny_logger::Channel;

let color: Color = Channel::Notice.get_color();
```

Colors can also be unconditionally enabled or disabled. This is especially useful when logging to something other than
the console.

```rust
tiny_logger::disable_color(); // Disable all colors/styling globally.
tiny_logger::enable_color();  // Enable all colors/styling globally.
```

## Streams

By default, every channel is written to `StdOut`. However, you can configure the stream per channel. This requires a
little bit of care with respect to types. The relevant setter is the method
`Channel::set_stream(&self, stream: Arc<Mutex<dyn Write + Send>>)`, which takes a trait object wrapped in an
`Arc<Mutex>`. The point of the `Arc<Mutex>` is so that a single object instance implementing `Write` can be shared
among multiple channels (as we do with `StdOut`). If you declare a concrete type, you have to erase the type when
supplying the object to `Channel::set_stream`.

Here is an illustrative example.

```rust
use std::sync::{Arc, Mutex};
use std::io::{Write};
use tiny_logger::{Channel, set_verbosity, log};

// Create a buffer to capture log output. Note that `Vec<u8>` implements `Write + Send`.
let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
// Now set the stream for the info channel. Notice the type erasure required here!
Channel::Info.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
// Set the verbosity to emit a message.
set_verbosity(1);
// Emit a message to the info channel.
log(Channel::Info, 0, "This is an Info message.");
// Check the buffer contents by inspecting the underlying `Vec<u8>`.
// Note that the type of `logged_output` is inferred to be `MutexGuard<Vec<u8>>`,
// reflecting the declared type of `buffer`.
let logged_output = buffer.lock().unwrap();
// The output is colored by default, so we use `from_utf8_lossy` to tolerate
// ANSI color codes. (We could instead turn colored output off with
// `tiny_logger::disable_color()`, but it's useful to see this way, too.)
let logged_string = String::from_utf8_lossy(&*logged_output);
assert!(logged_string.contains("This is an Info message.")); // Success!
```

When logging to something other than a console, such as a file or a string buffer, you will probably want to disable
colored/styled output globally with `tiny_logger::disable_color()`.

# Potential for Improvement

These are features that this library _does not_ have. There are lots of other great logging libraries, and this 
library is not trying to compete with more feature-full alternatives. Still, if you feel inspired to contribute... 

Some ideas for additional features / improvements:

 - More intelligent handling of output streams. E.g. automatic enable/disable of colors depending on stream type, 
   multiple streams per channel, etc.
 - Configurable formatting: date stamps and so forth
 - Smarter flushing. Right now we flush after each call.
 - Shorthand functions: `log_info("info message")` might have a default level and log to the info channel
 - Filtered views (hide certain messages rather than not emitting them at all)
 - Shorthand functions for logging to file(s)
 - interface with the [`log` crate](https://crates.io/crates/log) 
 - Error-like channels should output to StdErr
 - Relevant parts of `yansi` should be re-exported.
 - `yansi` should be an optional feature.

# Authorship and License

Copyright (c) 2024 Robert Jacobson

This code is released under either the MIT License or the Apache License 2.0, at your option.

MIT License:
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

1. The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

2. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Apache License 2.0:
Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
