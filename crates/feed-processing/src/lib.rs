mod worker;

pub use worker::*;

/// A trait for processing feed data with configurable input and output types
///
/// # Type Parameters
/// * `I` - The input type that will be processed
/// * `O` - The output type that will be produced after processing
///
/// This trait provides a common interface for feed processors that transform
/// input data of type `I` into output data of type `O`. The processing may
/// fail, which is why the output is wrapped in an `Option`.
pub trait FeedProcessor<I, O> {
    /// Processes the given input and attempts to produce an output
    ///
    /// # Parameters
    /// * `input` - The input data to process, of type `I`
    ///
    /// # Returns
    /// * `Option<O>` - The processed output if successful (`Some`), or `None` if processing failed
    fn process(&mut self, input: &I) -> Option<O>;
}
