//! An ergonomic and easy-to-integrate implementation of the
//! [GDB Remote Serial Protocol](https://sourceware.org/gdb/onlinedocs/gdb/Remote-Protocol.html#Remote-Protocol)
//! in Rust, with full `#![no_std]` support.
//!
//! ## Feature flags
//!
//! By default, both the `std` and `alloc` features are enabled.
//!
//! When using `gdbstub` in `#![no_std]` contexts, make sure to set
//! `default-features = false`.
//!
//! - `alloc`
//!     - Implement `Connection` for `Box<dyn Connection>`.
//!     - Log outgoing packets via `log::trace!` using a heap-allocated output
//!       buffer.
//!     - Provide built-in implementations for certain protocol features:
//!         - Use a heap-allocated packet buffer in `GdbStub` (if none is
//!           provided via `GdbStubBuilder::with_packet_buffer`).
//!         - (Monitor Command) Use a heap-allocated output buffer in
//!           `ConsoleOutput`.
//! - `std` (implies `alloc`)
//!     - Implement `Connection` for [`TcpStream`](std::net::TcpStream) and
//!       [`UnixStream`](std::os::unix::net::UnixStream).
//!     - Implement [`std::error::Error`] for `gdbstub::Error`.
//!     - Add a `TargetError::Io` error variant to simplify I/O Error handling
//!       from `Target` methods.
//!
//! ## Getting Started
//!
//! This section provides a brief overview of the key traits and types used in
//! `gdbstub`, and walks though the basic steps required to integrate `gdbstub`
//! into a project.
//!
//! At a high level, there are only three things that are required to get up and
//! running with `gdbstub`: a [`Connection`](#the-connection-trait), a
//! [`Target`](#the-target-trait), and a [event loop](#the-event-loop).
//!
//! > _Note:_ I _highly recommended_ referencing some of the
//! [examples](https://github.com/daniel5151/gdbstub/blob/master/README.md#examples)
//! listed in the project README when integrating `gdbstub` into a project for
//! the first time.
//!
//! > In particular, the in-tree
//! [`armv4t`](https://github.com/daniel5151/gdbstub/tree/master/examples/armv4t)
//! example contains basic implementations off almost all protocol extensions,
//! making it an incredibly valuable reference when implementing protocol
//! extensions.
//!
//! ### The `Connection` Trait
//!
//! First things first: `gdbstub` needs some way to communicate with a GDB
//! client. To facilitate this communication, `gdbstub` uses a custom
//! [`Connection`] trait.
//!
//! `Connection` is automatically implemented for common `std` types such as
//! [`TcpStream`](std::net::TcpStream) and
//! [`UnixStream`](std::os::unix::net::UnixStream).
//!
//! If you're using `gdbstub` in a `#![no_std]` environment, `Connection` will
//! most likely need to be manually implemented on top of whatever in-order,
//! serial, byte-wise I/O your particular platform has available (e.g:
//! putchar/getchar over UART, using an embedded TCP stack, etc.).
//!
//! One common way to start a remote debugging session is to simply wait for a
//! GDB client to connect via TCP:
//!
//! ```rust
//! use std::io;
//! use std::net::{TcpListener, TcpStream};
//!
//! fn wait_for_gdb_connection(port: u16) -> io::Result<TcpStream> {
//!     let sockaddr = format!("localhost:{}", port);
//!     eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);
//!     let sock = TcpListener::bind(sockaddr)?;
//!     let (stream, addr) = sock.accept()?;
//!
//!     // Blocks until a GDB client connects via TCP.
//!     // i.e: Running `target remote localhost:<port>` from the GDB prompt.
//!
//!     eprintln!("Debugger connected from {}", addr);
//!     Ok(stream) // `TcpStream` implements `gdbstub::Connection`
//! }
//! ```
//!
//! ### The `Target` Trait
//!
//! The [`Target`](target::Target) trait describes how to control and modify
//! a system's execution state during a GDB debugging session, and serves as the
//! primary bridge between `gdbstub`'s generic GDB protocol implementation and a
//! specific target's project/platform-specific code.
//!
//! At a high level, the `Target` trait is a collection of user-defined handler
//! methods that the GDB client can invoke via the GDB remote serial protocol.
//! For example, the `Target` trait includes methods to read/write
//! registers/memory, start/stop execution, etc...
//!
//! **`Target` is the most important trait in `gdbstub`, and must be implemented
//! by anyone integrating `gdbstub` into their project!**
//!
//! Please refer to the [`target` module documentation](target) for in-depth
//! instructions on how to implement [`Target`](target::Target) for a particular
//! platform.
//!
//! ## The Event Loop
//!
//! Once a [`Connection`](#the-connection-trait) has been established and
//! [`Target`](#the-target-trait) has been all wired up, all that's left is to
//! wire things up, and decide what kind of event loop to use!
//!
//! First things first, let's get an instance of `GdbStub` ready to run:
//!
//! ```rust,ignore
//! // Set-up a valid `Target`
//! let mut target = MyTarget::new()?; // implements `Target`
//!
//! // Establish a `Connection`
//! let connection: TcpStream = wait_for_gdb_connection(9001);
//!
//! // Create a new `gdbstub::GdbStub` using the established `Connection`.
//! let mut debugger = gdbstub::GdbStub::new(connection);
//! ```
//!
//! Cool, but how do you actually start the debugging session?
//!
//! ### `GdbStub::run_blocking`: The quick and easy way to get up and running
//! with `gdbstub`
//!
//! If you're running on a hosted system with threads to spare, the quickest way
//! to get up and running with `gdbstub` is by using the
//! [`GdbStub::run_blocking`] API alongside the
//! [`BlockingEventLoop`](crate::gdbstub_run_blocking::BlockingEventLoop) trait.
//!
//! A basic integration might look something like this:
//!
//! ```rust,ignore
//! use gdbstub::gdbstub_run_blocking;
//! use gdbstub::ConnectionExt; // note the use of `ConnectionExt` vs. `Connection`
//! use gdbstub::target::ext::base::multithread::ThreadStopReason;
//! use gdbstub::target::ext::base::singlethread::StopReason;
//!
//! enum MyGdbBlockingEventLoop {}
//!
//! impl gdbstub_run_blocking::BlockingEventLoop for MyGdbBlockingEventLoop {
//!     type Target = MyTarget;
//!     type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;
//!
//!     /// Invoked immediately after the target's `resume` method has been
//!     /// called. The implementation should block until either the target
//!     /// reports a stop reason, or if new data was sent over the connection.
//!     fn wait_for_stop_reason(
//!         target: &mut MyTarget,
//!         conn: &mut Self::Connection,
//!     ) -> Result<
//!         gdbstub_run_blocking::Event<u32>,
//!         gdbstub_run_blocking::WaitForStopReasonError<
//!             <Self::Target as Target>::Error,
//!             std::io::Error,
//!         >,
//!     > {
//!         // the specific mechanism to "select" between incoming data and target
//!         // events will depend on your project's architecture.
//!         //
//!         // some examples of how you might implement this method include: `epoll`,
//!         // `select!` across multiple event channels, periodic polling, etc...
//!         let event = match target.run_and_check_for_incoming_data(conn) {
//!             MyTargetEvent::IncomingData => {
//!                 let byte = conn
//!                     .read() // method provided by the `ConnectionExt` trait
//!                     .map_err(gdbstub_run_blocking::WaitForStopReasonError::Connection)?;
//!
//!                 gdbstub_run_blocking::Event::IncomingData(byte)
//!             }
//!             MyTargetEvent::StopReason(reason) => {
//!                 gdbstub_run_blocking::Event::TargetStopped(
//!                     target_event_to_gdb_event(reason)
//!                 )
//!             }
//!         };
//!
//!         Ok(event)
//!     }
//!
//!     /// Invoked when the GDB client sends a Ctrl-C interrupt. The
//!     /// implementation should handle the interrupt request + return an
//!     /// appropriate stop reason to report back to the GDB client, or return
//!     /// `None` if the interrupt should be ignored.
//!     fn on_interrupt(
//!         target: &mut MyTarget,
//!     ) -> Result<Option<ThreadStopReason<u32>>, <MyTarget as Target>::Error> {
//!         target.stop_in_response_to_ctrl_c_interrupt()?;
//!         // a pretty typical stop reason in response to a Ctrl-C interrupt is to
//!         // report a "Signal::SIGINT".
//!         Ok(Some(StopReason::Signal(Signal::SIGINT).into()))
//!     }
//! }
//!
//! fn gdb_event_loop_thread(
//!     debugger: gdbstub::GdbStub<MyTarget, Box<dyn ConnectionExt<Error = std::io::Error>>>,
//!     mut target: MyTarget
//! ) {
//!     match debugger.run_blocking::<MyGdbBlockingEventLoop>(&mut target) {
//!         Ok(disconnect_reason) => match disconnect_reason {
//!             DisconnectReason::Disconnect => {
//!                 println!("Client disconnected")
//!             }
//!             DisconnectReason::TargetExited(code) => {
//!                 println!("Target exited with code {}", code)
//!             }
//!             DisconnectReason::TargetTerminated(sig) => {
//!                 println!("Target terminated with signal {}", sig)
//!             }
//!             DisconnectReason::Kill => println!("GDB sent a kill command"),
//!         },
//!         Err(gdbstub::GdbStubError::TargetError(e)) => {
//!             println!("target encountered a fatal error: {}", e)
//!         }
//!         Err(e) => {
//!             println!("gdbstub encountered a fatal error: {}", e)
//!         }
//!     }
//! }
//! ```
// use an explicit doc attribute to avoid automatic rustfmt wrapping
#![doc = "### `GdbStubStateMachine`: Driving `gdbstub` in an async event loop / via interrupt handlers"]
//!
//! `GdbStub::run_blocking` requires that the target implement the
//! [`BlockingEventLoop`](crate::gdbstub_run_blocking::BlockingEventLoop) trait,
//! which as the name implies, uses _blocking_ IO when handling certain events.
//! Blocking the thread is a totally reasonable approach in most
//! implementations, as one can simply spin up a separate thread to run the GDB
//! stub (or in certain emulator implementations, run the emulator as part of
//! the `wait_for_stop_reason` method).
//!
//! Unfortunately, this blocking behavior can be a non-starter when integrating
//! `gdbstub` in projects that don't support / wish to avoid the traditional
//! thread-based execution model, such as projects using `async/await`, or
//! bare-metal, `no_std` projects running on embedded hardware.
//!
//! In these cases, `gdbstub` provides access to the underlying
//! [`GdbStubStateMachine`](state_machine::GdbStubStateMachine) API, which gives
//! implementations full control over the GDB stub's "event loop". This API
//! requires implementations to "push" data to the `gdbstub` implementation
//! whenever new data becomes available (e.g: when a UART interrupt handler
//! receives a byte, when the target hits a breakpoint, etc...), as opposed to
//! the `GdbStub::run_blocking` API, which "pulls" these events in a blocking
//! manner.
//!
//! See the [`GdbStubStateMachine`](state_machine::GdbStubStateMachine) docs for
//! more details on how to use this API.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
// Primarily due to rust-lang/rust#8995
//
// If this ever gets fixed, it's be possible to rewrite complex types using inherent associated type
// aliases.
//
// For example, instead of writing this monstrosity:
//
// Result<Option<ThreadStopReason<<Self::Arch as Arch>::Usize>>, Self::Error>
//
// ...it could be rewritten as:
//
// type StopReason = ThreadStopReason<<Self::Arch as Arch>::Usize>>;
//
// Result<Option<StopReason>, Self::Error>
#![allow(clippy::type_complexity)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
extern crate log;

mod connection;
mod gdbstub_impl;
mod protocol;
mod util;

#[doc(hidden)]
pub mod internal;

pub mod arch;
pub mod common;
pub mod target;

pub use connection::{Connection, ConnectionExt};
pub use gdbstub_impl::*;

/// (Internal) The fake Tid that's used when running in single-threaded mode.
// SAFETY: 1 is clearly non-zero.
const SINGLE_THREAD_TID: common::Tid = unsafe { common::Tid::new_unchecked(1) };
/// (Internal) The fake Pid reported to GDB (since `gdbstub` only supports
/// debugging a single process).
const FAKE_PID: common::Pid = unsafe { common::Pid::new_unchecked(1) };
