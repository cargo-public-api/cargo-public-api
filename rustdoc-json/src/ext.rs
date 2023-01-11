#![allow(clippy::disallowed_methods, clippy::module_name_repetitions)]
pub trait CommandExt {
    fn log(&self);
    fn status_and_log(&mut self) -> std::io::Result<std::process::ExitStatus>;
    fn output_and_log(&mut self) -> std::io::Result<std::process::Output>;
}

impl CommandExt for std::process::Command {
    fn status_and_log(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.log();
        self.status()
    }

    fn output_and_log(&mut self) -> std::io::Result<std::process::Output> {
        self.log();
        self.output()
    }


    #[cfg(not(feature = "log"))]
    fn log(&self) {}

    #[cfg(feature = "log")]
    fn log(&self) {
        if let Some(cwd) = self.get_current_dir() {
            log::debug!("running `{} {:?}`", cwd.display(), self);
        } else {
            log::debug!("running `{:?}`", self);
        }
    }
}
