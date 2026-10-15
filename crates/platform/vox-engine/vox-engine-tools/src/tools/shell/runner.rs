//! Process mechanics for the bash tool: spawn and a bounded wait
//! (ADR-0011).
//!
//! The child is kept here so it can be killed, and the pipes are
//! drained by the readers in [`pipes`]: a child that fills a pipe
//! buffer while the parent waits would deadlock, and a descendant
//! that outlives the command must not hang the session.

use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::tools::shell::pipes;

/// How often the waiter checks whether the child exited.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Grace for trailing pipe bytes once the command's life is over.
///
/// The command's own output was written before it exited (or was
/// killed), so everything attributable to it is in the pipe
/// buffer within scheduling jitter. A descendant that holds the
/// write end open is not the command: waiting longer would let
/// `sh -c "sleep 30 & echo started"` hold the tool for the whole
/// budget although its answer was ready immediately.
const OUTPUT_GRACE: Duration = Duration::from_millis(250);

/// The outcome of one finished or killed command.
pub(crate) enum Outcome {
    /// The process exited; `code` is -1 when the status carried
    /// none. `complete` is false when a pipe was still open at the
    /// budget deadline - a background child holding the write end.
    Exited {
        code: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        complete: bool,
    },
    /// The timeout passed before the process exited.
    TimedOut,
}

/// Run `program` in `cwd`, waiting at most `timeout` milliseconds.
pub(crate) fn run(
    program: &str,
    rest: &[String],
    cwd: &std::path::Path,
    timeout_ms: u64,
) -> std::io::Result<Outcome> {
    let mut child = Command::new(program)
        .args(rest)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let out_pipe = child.stdout.take();
    let err_pipe = child.stderr.take();
    let rx = pipes::spawn_readers(out_pipe, err_pipe);

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code().unwrap_or(-1);
                // The command is done: everything it produced is
                // already in the pipe buffers, so a short grace is
                // enough. Waiting for the full deadline here would
                // hand the budget to any descendant holding the
                // write end open.
                let (stdout, stderr, complete) = pipes::collect(&rx, Instant::now() + OUTPUT_GRACE);
                return Ok(Outcome::Exited {
                    code,
                    stdout,
                    stderr,
                    complete,
                });
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            // Timeout kills the process. The pipe readers are not
            // waited on beyond a short grace, so a child of the
            // child cannot extend the call; the captured streams of
            // a killed command are not part of the answer.
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = pipes::collect(&rx, Instant::now() + OUTPUT_GRACE);
                return Ok(Outcome::TimedOut);
            }
            Err(_) => return Ok(Outcome::TimedOut),
        }
    }
}
