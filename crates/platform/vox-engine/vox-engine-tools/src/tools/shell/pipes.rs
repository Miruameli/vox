//! Bounded pipe draining for the bash tool (ADR-0011).
//!
//! Readers report cumulative snapshots over a channel and are
//! never joined: a background process that outlives the command
//! and holds a pipe write end open must not be able to hang the
//! session. Every wait here is bounded by a caller-supplied
//! deadline.

use std::io::Read;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

/// Read batch size; snapshots are cumulative so a blocked reader
/// still gets its bytes to the caller.
const READ_CHUNK: usize = 8192;

/// Per-stream read bound, so a runaway writer cannot buffer
/// unbounded memory in a detached reader. The tool layer applies
/// its own, tighter cap to what the model actually sees.
const READ_BOUND: usize = 1024 * 1024;

/// Which pipe a snapshot belongs to.
#[derive(Clone, Copy)]
pub(crate) enum Stream {
    Stdout,
    Stderr,
}

/// One reader's cumulative report.
pub(crate) struct Snapshot {
    pub(crate) stream: Stream,
    pub(crate) buffer: Vec<u8>,
    pub(crate) complete: bool,
}

/// Spawn readers for both pipes of a child and return the channel
/// they report on.
pub(crate) fn spawn_readers(
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
) -> Receiver<Snapshot> {
    let (tx, rx) = mpsc::channel();
    let out_tx = tx.clone();
    thread::spawn(move || read_bounded(stdout, Stream::Stdout, out_tx));
    thread::spawn(move || read_bounded(stderr, Stream::Stderr, tx));
    rx
}

/// Drain `pipe` into cumulative snapshots until the pipe ends or
/// the read bound is reached.
///
/// A snapshot goes out after every read batch: a reader blocked on
/// a pipe a descendant still holds must still deliver the bytes it
/// already saw.
fn read_bounded<R: Read + Send + 'static>(
    pipe: Option<R>,
    stream: Stream,
    report: Sender<Snapshot>,
) {
    let mut buffer = Vec::new();
    let mut complete = true;
    if let Some(mut pipe) = pipe {
        let mut chunk = [0u8; READ_CHUNK];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&chunk[..n]);
                    let _ = report.send(Snapshot {
                        stream,
                        buffer: buffer.clone(),
                        complete: false,
                    });
                    if buffer.len() >= READ_BOUND {
                        complete = false;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
    let _ = report.send(Snapshot {
        stream,
        buffer,
        complete,
    });
}

/// Gather the latest snapshot of each stream before `deadline`.
///
/// Returns as soon as both streams report completion; a stream a
/// detached descendant still holds arrives as whatever the reader
/// last saw, marked incomplete.
pub(crate) fn collect(rx: &Receiver<Snapshot>, deadline: Instant) -> (Vec<u8>, Vec<u8>, bool) {
    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    let mut out_done = false;
    let mut err_done = false;
    while !(out_done && err_done) {
        let now = Instant::now();
        if now >= deadline {
            return (stdout, stderr, false);
        }
        match rx.recv_timeout(deadline - now) {
            Ok(snapshot) => match snapshot.stream {
                Stream::Stdout => {
                    stdout = snapshot.buffer;
                    out_done = snapshot.complete;
                }
                Stream::Stderr => {
                    stderr = snapshot.buffer;
                    err_done = snapshot.complete;
                }
            },
            // All senders dropped or the budget ran out.
            Err(_) => return (stdout, stderr, false),
        }
    }
    (stdout, stderr, true)
}
