#[derive(Debug, Fail)]
pub enum MinistatFailure {
    #[fail(display = "Invalid data on line {} of {}", line_no, file)]
    InvalidData { line_no: usize, file: String },
    #[fail(display = "'{}' is not a valid column (must be at least 1)", provided_column)]
    InvalidColumn { provided_column: String },
    #[fail(display = "'{}' is not a valid confidence (must be one of 80, 90, 95, 98, 99 and 99.5)", provided_confidence)]
    InvalidConfidence { provided_confidence: String },
    #[fail(display = "Dataset {} must contain at least 3 datapoints. (Perhaps there was not enough data in the column you selected?)", file)]
    InsufficientData { file: String },
    #[fail(display = "Unable to create a plot for this data")]
    NoPlotPossible,
    #[fail(display = "Too many datasets. You may have at most 7; you had {}", dataset_count)]
    TooManyDatasets { dataset_count: usize },
}
