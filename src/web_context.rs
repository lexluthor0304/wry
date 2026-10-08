// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(gtk)]
use crate::webkitgtk::WebContextImpl;

#[cfg(gtk)]
use std::time::Duration;
use std::{
  collections::HashSet,
  path::{Path, PathBuf},
};

/// A context that is shared between multiple [`WebView`]s.
///
/// A browser would have a context for all the normal tabs and a different context for all the
/// private/incognito tabs.
///
/// ## Platform-specific
///
/// - **Linux**: [`WebContextExtUnix::new_with_memory_pressure_settings`] creates a context whose
///   WebKitGTK web processes use custom memory-pressure settings, and
///   [`set_default_memory_pressure_settings`] gives every context created with
///   [`WebContext::new`] afterwards the same.
///
/// # Warning
///
/// If [`WebView`] is created by a WebContext. Dropping `WebContext` will cause [`WebView`] lose
/// some actions like custom protocol on Mac. Please keep both instances when you still wish to
/// interact with them.
///
/// [`WebView`]: crate::WebView
#[derive(Debug)]
pub struct WebContext {
  data_directory: Option<PathBuf>,
  #[allow(dead_code)] // It's not needed on Windows and macOS.
  pub(crate) os: WebContextImpl,
  #[allow(dead_code)] // It's not needed on Windows and macOS.
  pub(crate) custom_protocols: HashSet<String>,
}

impl WebContext {
  /// Create a new [`WebContext`].
  ///
  /// - `data_directory`: Whether the WebView window should have a custom user data path.
  ///   This is useful in Windows when a bundled application can't have the webview data inside `Program Files`.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Webview instances with different `CoreWebView2EnvironmentOptions` must have different `data_directory`s [^1]
  ///
  /// [^1]: <https://learn.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2environment.createcorewebview2controllerasync?view=webview2-dotnet-1.0.3719.77#:~:text=WebView%20creation%20fails%20if%20a%20running%20instance%20using%20the%20same%20user%20data%20folder%20exists%2C%20and%20the%20Environment%20objects%20have%20different%20CoreWebView2EnvironmentOptions.>
  pub fn new(data_directory: Option<PathBuf>) -> Self {
    Self {
      os: WebContextImpl::new(data_directory.as_deref()),
      data_directory,
      custom_protocols: Default::default(),
    }
  }

  #[cfg(gtk)]
  pub(crate) fn new_ephemeral() -> Self {
    Self {
      os: WebContextImpl::new_ephemeral(),
      data_directory: None,
      custom_protocols: Default::default(),
    }
  }

  /// A reference to the data directory the context was created with.
  pub fn data_directory(&self) -> Option<&Path> {
    self.data_directory.as_deref()
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  pub(crate) fn register_custom_protocol(&mut self, name: String) -> Result<(), crate::Error> {
    if self.is_custom_protocol_registered(&name) {
      return Err(crate::Error::ContextDuplicateCustomProtocol(name));
    }
    self.custom_protocols.insert(name);
    Ok(())
  }

  /// Check if a custom protocol has been registered on this context.
  pub fn is_custom_protocol_registered(&self, name: &str) -> bool {
    self.custom_protocols.contains(name)
  }

  /// Set if this context allows automation.
  ///
  /// **Note:** This is currently only enforced on Linux, and has the stipulation that
  /// only 1 context allows automation at a time.
  pub fn set_allows_automation(&mut self, flag: bool) {
    self.os.set_allows_automation(flag);
  }
}

impl Default for WebContext {
  fn default() -> Self {
    Self::new(None)
  }
}

/// WebKitGTK memory-pressure settings for the web processes of a [`WebContext`].
///
/// WebKitGTK measures the memory of every web process once per `poll_interval` and, when it
/// exceeds a fraction of `memory_limit_mb`, releases non-critical memory (the conservative
/// policy), also releases critical memory (the strict policy) or kills the process. Every field
/// left as `None` keeps WebKitGTK's default, listed below. WebKitGTK does not accept values
/// outside the ranges below: it logs a critical warning and keeps the previous value.
///
/// See WebKitGTK's [`WebKitMemoryPressureSettings`](https://webkitgtk.org/reference/webkit2gtk/stable/struct.MemoryPressureSettings.html)
/// and [`WebContextExtUnix::new_with_memory_pressure_settings`].
#[cfg(gtk)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoryPressureSettings {
  /// Memory limit, in MB, that the thresholds are fractions of. Must be bigger than 0.
  ///
  /// WebKitGTK's default is the system's RAM size, with a maximum of 3 GB.
  pub memory_limit_mb: Option<u32>,
  /// Fraction of the memory limit from which the conservative policy releases non-critical
  /// memory. Must be bigger than 0, smaller than 1 and smaller than `strict_threshold`.
  ///
  /// WebKitGTK's default is 0.33.
  pub conservative_threshold: Option<f64>,
  /// Fraction of the memory limit from which the strict policy also releases critical memory.
  /// Must be bigger than `conservative_threshold`, smaller than 1 and, when `kill_threshold` is
  /// not 0, smaller than it.
  ///
  /// WebKitGTK's default is 0.5.
  pub strict_threshold: Option<f64>,
  /// Fraction of the memory limit from which the web process is killed. 0 never kills it; any
  /// other value must be bigger than `strict_threshold` and may exceed 1.
  ///
  /// WebKitGTK's default is 0 (never).
  pub kill_threshold: Option<f64>,
  /// Period between two memory measurements. Must be longer than zero.
  ///
  /// WebKitGTK's default is 30 seconds.
  pub poll_interval: Option<Duration>,
}

/// Additional methods on `WebContext` that are specific to Linux.
#[cfg(gtk)]
pub trait WebContextExtUnix: Sized {
  /// Create a new [`WebContext`] whose web processes use the given WebKitGTK memory-pressure
  /// settings.
  ///
  /// WebKitGTK takes these settings only when the `WebKitWebContext` is constructed (its
  /// `memory-pressure-settings` property is construct-only), so they cannot be changed on an
  /// existing context, and they apply to every webview created with this one. They affect the
  /// web processes only, not the network process. Webviews created with
  /// [`WebViewBuilder::with_incognito`](crate::WebViewBuilder::with_incognito) use their own
  /// ephemeral context and keep WebKitGTK's defaults.
  ///
  /// See [`WebContext::new`] for `data_directory`.
  ///
  /// # Example
  ///
  /// Let a web process use half of a 16 GB machine before WebKitGTK's strict policy starts
  /// (with the default limit of 3 GB it starts at 1.5 GB):
  ///
  /// ```no_run
  /// use wry::{MemoryPressureSettings, WebContext, WebContextExtUnix};
  ///
  /// let context = WebContext::new_with_memory_pressure_settings(
  ///   None,
  ///   MemoryPressureSettings {
  ///     memory_limit_mb: Some(8192),
  ///     ..Default::default()
  ///   },
  /// );
  /// ```
  fn new_with_memory_pressure_settings(
    data_directory: Option<PathBuf>,
    memory_pressure_settings: MemoryPressureSettings,
  ) -> Self;
}

#[cfg(gtk)]
impl WebContextExtUnix for WebContext {
  fn new_with_memory_pressure_settings(
    data_directory: Option<PathBuf>,
    memory_pressure_settings: MemoryPressureSettings,
  ) -> Self {
    Self {
      os: WebContextImpl::new_with_memory_pressure_settings(
        data_directory.as_deref(),
        Some(&memory_pressure_settings),
      ),
      data_directory,
      custom_protocols: Default::default(),
    }
  }
}

/// The process-wide default of [`set_default_memory_pressure_settings`].
#[cfg(gtk)]
static DEFAULT_MEMORY_PRESSURE_SETTINGS: std::sync::RwLock<Option<MemoryPressureSettings>> =
  std::sync::RwLock::new(None);

/// Set the WebKitGTK memory-pressure settings that every [`WebContext`] created afterwards with
/// [`WebContext::new`] (or `WebContext::default()`) applies to its web processes, which covers
/// the webviews built without an explicit context and the contexts a framework such as Tauri
/// creates on the application's behalf.
///
/// Call it before any `WebContext` or `WebView` is created: WebKitGTK takes the settings only
/// when a `WebKitWebContext` is constructed, so a context created earlier keeps the settings it
/// was built with. A context created with
/// [`WebContextExtUnix::new_with_memory_pressure_settings`] uses its own settings instead, and
/// the ephemeral context of a webview created with
/// [`WebViewBuilder::with_incognito`](crate::WebViewBuilder::with_incognito) keeps WebKitGTK's
/// defaults. Setting `MemoryPressureSettings::default()` (every field `None`) returns later
/// contexts to WebKitGTK's defaults.
///
/// # Example
///
/// Let the web processes use half of a 16 GB machine before WebKitGTK's strict policy starts
/// (with the default limit of 3 GB it starts at 1.5 GB), whichever code creates the context:
///
/// ```no_run
/// use wry::{set_default_memory_pressure_settings, MemoryPressureSettings};
///
/// set_default_memory_pressure_settings(MemoryPressureSettings {
///   memory_limit_mb: Some(8192),
///   ..Default::default()
/// });
/// ```
#[cfg(gtk)]
pub fn set_default_memory_pressure_settings(settings: MemoryPressureSettings) {
  *DEFAULT_MEMORY_PRESSURE_SETTINGS
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(settings);
}

/// The process-wide default set by [`set_default_memory_pressure_settings`], or `None` while
/// none was set and new contexts keep WebKitGTK's defaults.
#[cfg(gtk)]
pub fn default_memory_pressure_settings() -> Option<MemoryPressureSettings> {
  *DEFAULT_MEMORY_PRESSURE_SETTINGS
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(not(gtk))]
#[derive(Debug)]
pub(crate) struct WebContextImpl;

#[cfg(not(gtk))]
impl WebContextImpl {
  fn new(_: Option<&Path>) -> Self {
    Self
  }

  fn set_allows_automation(&mut self, _flag: bool) {}
}

#[cfg(all(test, gtk))]
mod tests {
  use super::{
    default_memory_pressure_settings, set_default_memory_pressure_settings, MemoryPressureSettings,
  };

  #[test]
  fn default_memory_pressure_settings_are_kept_process_wide() {
    assert_eq!(default_memory_pressure_settings(), None);
    let settings = MemoryPressureSettings {
      memory_limit_mb: Some(8192),
      ..Default::default()
    };
    set_default_memory_pressure_settings(settings);
    assert_eq!(default_memory_pressure_settings(), Some(settings));
    set_default_memory_pressure_settings(MemoryPressureSettings::default());
    assert_eq!(
      default_memory_pressure_settings(),
      Some(MemoryPressureSettings::default())
    );
  }
}
