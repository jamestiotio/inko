//! Functions for setting/getting environment and operating system data.
use crate::mem::{Array, Pointer, String as InkoString};
use crate::platform;
use crate::process::ProcessPointer;
use crate::runtime_error::RuntimeError;
use crate::scheduler::process::Thread;
use crate::state::State;
use std::env;
use std::path::PathBuf;

pub(crate) fn env_get(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    arguments: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let var_name = unsafe { InkoString::read(&arguments[0]) };
    let result = state
        .environment
        .get(var_name)
        .cloned()
        .unwrap_or_else(Pointer::undefined_singleton);

    Ok(result)
}

pub(crate) fn env_variables(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let names = state
        .environment
        .keys()
        .map(|key| {
            InkoString::alloc(state.permanent_space.string_class(), key.clone())
        })
        .collect();

    Ok(Array::alloc(state.permanent_space.array_class(), names))
}

pub(crate) fn env_home_directory(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    // Rather than performing all sorts of magical incantations to get the home
    // directory, we're just going to require that these environment variables
    // are set.
    let var = if cfg!(windows) {
        state.environment.get("USERPROFILE")
    } else {
        state.environment.get("HOME")
    };

    // If the home is explicitly set to an empty string we still ignore it,
    // because there's no scenario in which Some("") is useful.
    let result = var
        .cloned()
        .filter(|p| unsafe { !InkoString::read(p).is_empty() })
        .unwrap_or_else(Pointer::undefined_singleton);

    Ok(result)
}

pub(crate) fn env_temp_directory(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let path = canonalize(env::temp_dir().to_string_lossy().into_owned());

    Ok(InkoString::alloc(state.permanent_space.string_class(), path))
}

pub(crate) fn env_get_working_directory(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let path = env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map(canonalize)?;

    Ok(InkoString::alloc(state.permanent_space.string_class(), path))
}

pub(crate) fn env_set_working_directory(
    _: &State,
    _: &mut Thread,
    _: ProcessPointer,
    arguments: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let dir = unsafe { InkoString::read(&arguments[0]) };

    env::set_current_dir(dir)?;
    Ok(Pointer::nil_singleton())
}

pub(crate) fn env_arguments(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    Ok(Array::alloc(
        state.permanent_space.array_class(),
        state.arguments.clone(),
    ))
}

pub(crate) fn env_platform(
    _: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    Ok(Pointer::int(platform::operating_system()))
}

pub(crate) fn env_executable(
    state: &State,
    _: &mut Thread,
    _: ProcessPointer,
    _: &[Pointer],
) -> Result<Pointer, RuntimeError> {
    let path = env::current_exe()?.to_string_lossy().into_owned();

    Ok(InkoString::alloc(state.permanent_space.string_class(), path))
}

fn canonalize(path: String) -> String {
    PathBuf::from(&path)
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path)
}
