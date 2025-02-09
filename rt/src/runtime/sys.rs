use crate::mem::{ByteArray, String as InkoString};
use crate::process::ProcessPointer;
use crate::result::Result as InkoResult;
use crate::runtime::helpers::read_into;
use crate::scheduler::number_of_cores;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::slice;

fn stdio_for(value: i64) -> Stdio {
    match value {
        1 => Stdio::inherit(),
        2 => Stdio::piped(),
        _ => Stdio::null(),
    }
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_spawn(
    process: ProcessPointer,
    program: *const InkoString,
    args: *const *const InkoString,
    args_length: i64,
    env: *const *const InkoString,
    env_length: i64,
    stdin: i64,
    stdout: i64,
    stderr: i64,
    directory: *const InkoString,
) -> InkoResult {
    let program = InkoString::read(program);
    let args = slice::from_raw_parts(args, args_length as _);
    let env = slice::from_raw_parts(env, env_length as _);
    let directory = InkoString::read(directory);
    let mut cmd = Command::new(program);

    for &ptr in args {
        cmd.arg(InkoString::read(ptr as _));
    }

    for pair in env.chunks(2) {
        let key = InkoString::read(pair[0] as _);
        let val = InkoString::read(pair[1] as _);

        cmd.env(key, val);
    }

    cmd.stdin(stdio_for(stdin));
    cmd.stdout(stdio_for(stdout));
    cmd.stderr(stdio_for(stderr));

    if !directory.is_empty() {
        cmd.current_dir(directory);
    }

    process
        .blocking(|| cmd.spawn())
        .map(InkoResult::ok_boxed)
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_wait(
    process: ProcessPointer,
    child: *mut Child,
) -> InkoResult {
    process
        .blocking(|| (*child).wait())
        .map(|status| status.code().unwrap_or(0) as i64)
        .map(|status| InkoResult::ok(status as _))
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_try_wait(
    child: *mut Child,
) -> InkoResult {
    let child = &mut *child;

    child
        .try_wait()
        .map(|status| {
            InkoResult::ok({
                status.map(|s| s.code().unwrap_or(0)).unwrap_or(-1) as i64
            } as _)
        })
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdout_read(
    process: ProcessPointer,
    child: *mut Child,
    buffer: *mut ByteArray,
    size: i64,
) -> InkoResult {
    let child = &mut *child;
    let buff = &mut (*buffer).value;

    child
        .stdout
        .as_mut()
        .map(|stream| process.blocking(|| read_into(stream, buff, size)))
        .unwrap_or(Ok(0))
        .map(|size| InkoResult::ok(size as _))
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stderr_read(
    process: ProcessPointer,
    child: *mut Child,
    buffer: *mut ByteArray,
    size: i64,
) -> InkoResult {
    let child = &mut *child;
    let buff = &mut (*buffer).value;

    child
        .stderr
        .as_mut()
        .map(|stream| process.blocking(|| read_into(stream, buff, size)))
        .unwrap_or(Ok(0))
        .map(|size| InkoResult::ok(size as _))
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdin_write_bytes(
    process: ProcessPointer,
    child: *mut Child,
    input: *mut ByteArray,
) -> InkoResult {
    let child = &mut *child;
    let input = &(*input).value;

    child
        .stdin
        .as_mut()
        .map(|stream| process.blocking(|| stream.write(input)))
        .unwrap_or(Ok(0))
        .map(|size| InkoResult::ok(size as _))
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdin_write_string(
    process: ProcessPointer,
    child: *mut Child,
    input: *mut InkoString,
) -> InkoResult {
    let child = &mut *child;
    let input = InkoString::read(input);

    child
        .stdin
        .as_mut()
        .map(|stream| process.blocking(|| stream.write(input.as_bytes())))
        .unwrap_or(Ok(0))
        .map(|size| InkoResult::ok(size as _))
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdin_flush(
    process: ProcessPointer,
    child: *mut Child,
) -> InkoResult {
    let child = &mut *child;

    child
        .stdin
        .as_mut()
        .map(|stream| process.blocking(|| stream.flush()))
        .unwrap_or(Ok(()))
        .map(|_| InkoResult::none())
        .unwrap_or_else(InkoResult::io_error)
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdout_close(
    child: *mut Child,
) {
    (*child).stdout.take();
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stderr_close(
    child: *mut Child,
) {
    (*child).stderr.take();
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_stdin_close(
    child: *mut Child,
) {
    (*child).stdin.take();
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_child_process_drop(
    child: *mut Child,
) {
    drop(Box::from_raw(child));
}

#[no_mangle]
pub(crate) unsafe extern "system" fn inko_cpu_cores() -> i64 {
    number_of_cores() as i64
}
