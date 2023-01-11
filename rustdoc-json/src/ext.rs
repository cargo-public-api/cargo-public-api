#![doc(hidden)]

#![allow(clippy::disallowed_methods, clippy::module_name_repetitions)]
pub trait CommandExt {
    fn log(&self);
    fn status_and_log(&mut self) -> std::io::Result<std::process::ExitStatus>;
    fn output_and_log(&mut self) -> std::io::Result<std::process::Output>;
    fn spawn_and_log(&mut self) -> std::io::Result<std::process::Child>;
}

impl CommandExt for std::process::Command {
    #[track_caller]
    fn status_and_log(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.log();
        self.status()
    }

    #[track_caller]
    fn output_and_log(&mut self) -> std::io::Result<std::process::Output> {
        self.log();
        self.output()
    }

    #[track_caller]
    fn spawn_and_log(&mut self) -> std::io::Result<std::process::Child> {
        self.log();
        self.spawn()
    }

    #[cfg(not(feature = "log"))]
    fn log(&self) {}

    #[track_caller]
    #[cfg(feature = "log")]
    fn log(&self) {
        if let Some(cwd) = self.get_current_dir() {
            log::debug!("running `{} {:?}`", cwd.display(), self);
        } else {
            log::debug!("running `{:?}`", self);
        }
    }
}
