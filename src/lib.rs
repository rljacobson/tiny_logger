/*!

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

## Channels

`Channel` takes one of the values:  Critical, Error, Warning, Notice, Info, Debug, Trace. Messages emitted to a channel
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

*/

use std::{
  collections::HashMap,
  io::Write,
  sync::{RwLock, Mutex}
};
use std::sync::Arc;
use yansi::{Color, Paint, Painted};

/// The verbosity level type.
pub type LogLevel = i32;

// Global data
lazy_static::lazy_static! {
  static ref VERBOSITY: RwLock<LogLevel> = RwLock::new(0);

  static ref LOGGING_STREAMS: RwLock<HashMap<Channel, Arc<Mutex<dyn Write + Send>>>> = {
    let stdout_stream: Arc<Mutex<dyn Write + Send>> = Arc::new(Mutex::new(std::io::stdout())); // Create a single Arc<Mutex<Stdout>>
    let mut m: HashMap<Channel, Arc<Mutex<dyn Write + Send>>> = HashMap::new();
    m.insert(Channel::Critical, Arc::clone(&stdout_stream));
    m.insert(Channel::Error,    Arc::clone(&stdout_stream));
    m.insert(Channel::Warning,  Arc::clone(&stdout_stream));
    m.insert(Channel::Notice,   Arc::clone(&stdout_stream));
    m.insert(Channel::Info,     Arc::clone(&stdout_stream));
    m.insert(Channel::Debug,    Arc::clone(&stdout_stream));
    m.insert(Channel::Trace,    Arc::clone(&stdout_stream));
    RwLock::new(m)
  };

  static ref CHANNEL_COLORS: RwLock<HashMap<Channel, Color>> = {
    let mut m = HashMap::new();
    m.insert(Channel::Critical, Color::Red);
    m.insert(Channel::Error,    Color::BrightRed); // Using BrightRed for Error
    m.insert(Channel::Warning,  Color::Yellow);
    m.insert(Channel::Notice,   Color::Blue);
    m.insert(Channel::Info,     Color::Green);
    m.insert(Channel::Debug,    Color::BrightBlack);
    m.insert(Channel::Trace,    Color::Cyan);
    RwLock::new(m)
  };
}

/// Channels to which a log entry can be published.
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum Channel {
  Critical,
  Error,
  Warning,
  Notice,
  Info,
  Debug,
  Trace,
}

impl Channel {
  /// Fetch the current color for the label of this channel.
  pub fn get_color(&self) -> Color {
    let colors = CHANNEL_COLORS.read().unwrap();
    *colors.get(self).unwrap_or(&Color::White) // Provide a default color if not set
  }

  /// Set the color for the label of this channel.
  pub fn set_color(&self, color: Color) {
    let mut colors = CHANNEL_COLORS.write().unwrap();
    colors.insert(*self, color);
  }

  /// Get a string of the name of this channel formatted in color.
  pub fn painted_name(&self) -> Painted<String> {
    let channel_name: String = format!("{:?}", self); // Get the name of the channel
    let color: Color         = self.get_color(); // Get the associated color
    let mut name: Painted<String> = Paint::new(channel_name);
    name.style = color.into(); // Paint the channel name with the color
    name
  }

  /// Set a new logging stream for this channel.
  pub fn set_stream(&self, stream: Arc<Mutex<dyn Write + Send>>) {
    let mut streams = LOGGING_STREAMS.write().unwrap();
    streams.insert(*self, stream); // Update the stream for the channel
  }
}

/// Set the global verbosity level.
pub fn set_verbosity(new_value: LogLevel) {
  let mut verbosity = VERBOSITY.write().unwrap();
  *verbosity = new_value;
}

pub fn get_verbosity() -> LogLevel {
  *VERBOSITY.read().unwrap()
}

/// Unconditionally disable color/styling globally. Use this when logging to a file.
pub fn disable_color() {
  yansi::disable();
}

/// Unconditionally enable color/styling globally. This is the default.
pub fn enable_color() {
  yansi::enable();
}

/// Returns true if color/styling is enabled globally. Otherwise, returns false.
pub fn color_is_enabled() -> bool {
  yansi::is_enabled()
}

/// Log a `message` to the given `Channel` at the specified (verbosity) `LogLevel`.
/// Only emits a message if the global verbosity level is at least `level`.
pub fn log(channel: Channel, log_level: LogLevel, message: &str) {
  let msg = format!("{}: {}", channel.painted_name(), message);

  if *VERBOSITY.read().unwrap() >= log_level {
    // Fetch the appropriate logging stream for the channel
    let channel_streams = LOGGING_STREAMS.read().unwrap();
    if let Some(log_stream) = channel_streams.get(&channel) {
      let mut locked_stream = log_stream.lock().unwrap(); // Lock the stream
      let _ = locked_stream.write(msg.as_bytes());
      let _ = locked_stream.write(b"\n");
      locked_stream.flush().unwrap();
    }
    // Note: If there is no stream for the given channel, we just don't emit the message.
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use yansi::Color;

  #[test]
  fn test_channel_colors_initialization() {
    // Verify initial colors
    let colors = CHANNEL_COLORS.read().unwrap();
    assert_eq!(*colors.get(&Channel::Critical).unwrap(), Color::Red);
    assert_eq!(*colors.get(&Channel::Error).unwrap(), Color::BrightRed);
    assert_eq!(*colors.get(&Channel::Warning).unwrap(), Color::Yellow);
    assert_eq!(*colors.get(&Channel::Notice).unwrap(), Color::Blue);
    assert_eq!(*colors.get(&Channel::Info).unwrap(), Color::Green);
    assert_eq!(*colors.get(&Channel::Debug).unwrap(), Color::BrightBlack);
    assert_eq!(*colors.get(&Channel::Trace).unwrap(), Color::Cyan);
  }

  #[test]
  fn test_set_color() {
    // Change the color of the Info channel
    Channel::Info.set_color(Color::Magenta);
    {
      let colors = CHANNEL_COLORS.read().unwrap();
      assert_eq!(*colors.get(&Channel::Info).unwrap(), Color::Magenta);
    }
    // Set it back for subsequent tests
    Channel::Info.set_color(Color::Green);
  }

  #[test]
  fn test_get_color() {
    let critical_color = Channel::Critical.get_color();
    assert_eq!(critical_color, Color::Red);
  }

  #[test]
  fn test_painted_name() {
    let painted_name = Channel::Warning.painted_name();
    let expected = "Warning".to_string().paint(Color::Yellow).to_string();
    assert_eq!(painted_name.to_string(), expected);
  }

  #[test]
  fn test_set_and_get_verbosity() {
    // Set verbosity and check if it was set correctly
    set_verbosity(3);
    {
      let verbosity = VERBOSITY.read().unwrap();
      assert_eq!(*verbosity, 3);
    }
    // Reset to default for subsequent tests
    set_verbosity(0);
  }

  #[test]
  fn test_logging() {
    // Create a buffer to capture log output wrapped in Arc<Mutex<dyn Write>>
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    // Set the logging stream for each channel to our buffer
    Channel::Critical.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Error.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Warning.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Notice.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Info.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Debug.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));
    Channel::Trace.set_stream(Arc::clone(&(buffer.clone() as Arc<Mutex<dyn Write + Send>>)));

    // Set verbosity to 1
    set_verbosity(1);
    disable_color();
    // println!("Verbosity: {}", get_verbosity());

    // Log messages
    log(Channel::Critical, 3, "Critical error occurred!");            // Not emitted
    log(Channel::Error, 2, "This is an error message.");               // Not emitted
    log(Channel::Warning, 1, "Warning: Check your input.");            // Emitted
    log(Channel::Notice, 0, "Notice: This is informational.");         // Emitted
    log(Channel::Info, 1, "Info: Processing started.");                // Emitted
    log(Channel::Debug, 0, "Debug: Variable values are correct.");     // Emitted
    log(Channel::Trace, 0, "Trace: Step through the logic here.");     // Emitted

    // Check the buffer contents
    let logged_output = buffer.lock().unwrap(); // Get the underlying Vec<u8>

    // Verify which messages were emitted by checking the byte representation
    let logged_string = String::from_utf8_lossy(&*logged_output);
    print!("{}", logged_string);
    assert!(logged_string.contains("Warning: Check your input."));
    assert!(logged_string.contains("Notice: This is informational."));
    assert!(logged_string.contains("Info: Processing started."));
    assert!(logged_string.contains("Debug: Variable values are correct."));
    assert!(logged_string.contains("Trace: Step through the logic here."));

    // Verify that not emitted messages are not in the buffer
    assert!(!logged_string.contains("Critical error occurred!"));
    assert!(!logged_string.contains("This is an error message."));

    // Reset verbosity to default for subsequent tests.
    set_verbosity(0)
  }
}
