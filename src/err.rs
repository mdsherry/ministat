use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinistatFailure {
    #[error("Invalid data on line {} of {}", line_no, file)]
    InvalidData { line_no: usize, file: String },
    #[error("'{}' is not a valid column (must be at least 1)", provided_column)]
    InvalidColumn { provided_column: String },
    #[error("'{}' is not a valid confidence (must be one of 80, 90, 95, 98, 99 and 99.5)", provided_confidence)]
    InvalidConfidence { provided_confidence: String },
    #[error("Dataset {} must contain at least 3 datapoints. (Perhaps there was not enough data in the column you selected?)", file)]
    InsufficientData { file: String },
    #[error("Unable to create a plot for this data")]
    NoPlotPossible,
    #[error("Too many datasets. You may have at most 7; you had {}", dataset_count)]
    TooManyDatasets { dataset_count: usize },
}
