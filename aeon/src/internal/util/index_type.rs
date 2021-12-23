// TODO: Use procedural macros to generate implementations for most of this.

/// Structs that implement this type declare that they serve as indices
/// into some collection of `T` values.
pub trait IndexType<T, Collection>: Sized {
    /// Return a usize representation of this index.
    fn to_index(&self) -> usize;
    /// Try to create an index from a given usize in the context of the given collection.
    fn try_from(index: usize, collection: &Collection) -> Option<Self>;
    /// Same as `try_from`, but also includes error handling for string parsing.
    fn try_from_str(index: &str, collection: &Collection) -> Option<Self> {
        index
            .parse::<usize>()
            .ok()
            .and_then(|i| Self::try_from(i, collection))
    }
}
