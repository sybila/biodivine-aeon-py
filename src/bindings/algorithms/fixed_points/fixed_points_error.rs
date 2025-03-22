use thiserror::Error;

// TODO: ohtenkay - impl Debug for FixedPointsError
/// An error returned by a [FixedPoints] procedure.
#[derive(Error, Debug)]
pub enum FixedPointsError {
    #[error("no fixed points found")]
    NoFixedPointsFound,
    // #[error("operation cancelled")]
    // Cancelled(GraphColoredVerticesq),
    // #[error("steps limit exceeded")]
    // StepsLimitExceeded(GraphColoredVertices),
    // #[error("BDD size limit exceeded")]
    // BddSizeLimitExceeded(GraphColoredVertices),
    // #[error("subgraph set not compatible with the given graph or initial states")]
    // InvalidSubgraph,
}
