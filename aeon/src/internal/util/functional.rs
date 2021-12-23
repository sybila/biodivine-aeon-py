pub trait Functional: Sized {
    /// Apply can be used as a dedicated "initializer" for the given value.
    ///
    /// It takes ownership of the given and returns it back once the action
    /// has been applied. Note that it cannot change the output type (output of
    /// `action` is ignored.
    fn apply<F, R>(mut self, action: F) -> Self
    where
        F: FnOnce(&mut Self) -> R,
    {
        action(&mut self);
        self
    }

    /// And then applies a lambda to the given (owned) value.
    ///
    /// It can be used to apply quick modifications to values before/after returning them.
    fn and_then<F, R>(self, action: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        action(self)
    }

    /// Run an non-modifying action with the given value. The return value
    /// is dropped.
    fn also<F, R>(self, action: F) -> Self
    where
        F: FnOnce(&Self) -> R,
    {
        action(&self);
        self
    }

    /// Conditionally wrap item in `Some`.
    ///
    /// Note that this always evaluates the value in question and shouldn't be thus
    /// used when side-effects are important or items are large (compiler can figure
    /// out if the value is not used though).
    fn take_if<F>(self, test: F) -> Option<Self>
    where
        F: FnOnce(&Self) -> bool,
    {
        if test(&self) {
            Some(self)
        } else {
            None
        }
    }
}

impl<T: Sized> Functional for T {}
