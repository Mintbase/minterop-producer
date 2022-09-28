#[macro_export]
macro_rules! warn {
    ($($args:tt)+) => {
        tracing::warn!(
            target: "minterop",
            $($args)*,
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {
        tracing::info!(
            target: "minterop",
            $($args)*,
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($args:tt)+) => {
        tracing::error!(
            target: "minterop",
            $($args)*,
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {
        tracing::debug!(
            target: "minterop",
            $($args)*,
        )
    };
}

pub(crate) trait HandleNone {
    fn handle_none<F: Fn()>(&self, f: F);
}

impl<T> HandleNone for Option<T> {
    fn handle_none<F: Fn()>(&self, f: F) {
        if self.is_none() {
            f();
        }
    }
}

pub(crate) trait HandleErr<E> {
    fn handle_err<F: Fn(&E)>(&self, f: F);
}

impl<T, E> HandleErr<E> for Result<T, E> {
    fn handle_err<F: Fn(&E)>(&self, f: F) {
        if let Err(e) = self {
            f(e);
        }
    }
}
