use std::num::{ParseFloatError, ParseIntError};

use rayon::ThreadPoolBuildError;
use thiserror::Error;

/// All possible errors returned by this library and the libraries it depends
/// on.
// Based on `<https://nick.groenen.me/posts/rust-error-handling/#the-library-error-type>`
#[derive(Error, Debug)]
pub enum BedErrorPlus {
    #[allow(missing_docs)]
    #[error(transparent)]
    BedError(#[from] BedError),

    #[allow(missing_docs)]
    #[error(transparent)]
    ThreadPoolError(#[from] ThreadPoolBuildError),

    #[allow(missing_docs)]
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[allow(missing_docs)]
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[allow(missing_docs)]
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

/// All errors specific to this library.
#[derive(Error, Debug, Clone)]
pub enum BedError {
    #[allow(missing_docs)]
    #[error("Ill-formed BED file. BED file header is incorrect or length is wrong. '{0}'")]
    IllFormed(String),

    #[allow(missing_docs)]
    #[error(
        "Ill-formed BED file. BED file header is incorrect. Expected mode to be 0 or 1. '{0}'"
    )]
    BadMode(String),

    #[allow(missing_docs)]
    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),

    #[allow(missing_docs)]
    #[error("Multithreading resulted in panic(s)")]
    PanickedThread(),

    #[allow(missing_docs)]
    #[error("No individual observed for the SNP.")]
    NoIndividuals,

    #[allow(missing_docs)]
    #[error("Illegal SNP mean.")]
    IllegalSnpMean,

    #[allow(missing_docs)]
    #[error("Index to individual larger than the number of individuals. (Index value {0})")]
    IidIndexTooBig(isize),

    #[allow(missing_docs)]
    #[error("Index to SNP larger than the number of SNPs. (Index value {0})")]
    SidIndexTooBig(isize),

    #[allow(missing_docs)]
    #[error(
        "Length of iid_index ({0}) and sid_index ({1}) must match dimensions of output array \
         ({2},{3})."
    )]
    IndexMismatch(usize, usize, usize, usize),

    #[allow(missing_docs)]
    #[error("Indexes ({0},{1}) too big for files")]
    IndexesTooBigForFiles(usize, usize),

    #[allow(missing_docs)]
    #[error(
        "Subset: length of iid_index ({0}) and sid_index ({1}) must match dimensions of output \
         array ({2},{3})."
    )]
    SubsetMismatch(usize, usize, usize, usize),

    #[allow(missing_docs)]
    #[error("Cannot convert beta values to/from float 64")]
    CannotConvertBetaToFromF64,

    #[allow(missing_docs)]
    #[error("Cannot create Beta Dist with given parameters ({0},{1})")]
    CannotCreateBetaDist(f64, f64),

    #[allow(missing_docs)]
    #[error("Cannot use skipped metadata '{0}'")]
    CannotUseSkippedMetadata(String),

    #[allow(missing_docs)]
    #[error("Index starts at {0} but ends at {1}")]
    StartGreaterThanEnd(usize, usize),

    #[allow(missing_docs)]
    #[error("Step of zero not allowed")]
    StepZero,

    #[allow(missing_docs)]
    #[error("Index starts at {0} but count is {1}")]
    StartGreaterThanCount(usize, usize),

    #[allow(missing_docs)]
    #[error("Index ends at {0} but count is {1}")]
    EndGreaterThanCount(usize, usize),

    #[allow(missing_docs)]
    #[error("Adding new axis not allowed")]
    NewAxis,

    #[allow(missing_docs)]
    #[error("Expect 1-D NDArray SliceInfo")]
    NdSliceInfoNot1D,

    #[allow(missing_docs)]
    #[error("Expect {0} fields but find only {1} in '{2}'")]
    MetadataFieldCount(usize, usize, String),

    #[allow(missing_docs)]
    #[error("{0}_count values of {1} and {2} are inconsistent")]
    InconsistentCount(String, usize, usize),

    #[allow(missing_docs)]
    #[error("Expect bool arrays and vectors to be length {0}, not {1}")]
    BoolArrayVectorWrongLength(usize, usize),

    #[allow(missing_docs)]
    #[error("Expect ndarray of shape ({0}, {1}), but found shape ({2}, {3})")]
    InvalidShape(usize, usize, usize, usize),

    #[allow(missing_docs)]
    #[error("Can't write '{0}' metadata if some fields are None")]
    MetadataMissingForWrite(String),

    #[allow(missing_docs)]
    #[error("Unknown or bad sample file '{0}'")]
    UnknownOrBadSampleFile(String),

    #[allow(missing_docs)]
    #[error("The registry of sample files is invalid")]
    SampleRegistryProblem(),

    #[allow(missing_docs)]
    #[error("Samples construction failed with error: {0}")]
    SamplesConstructionFailed(String),

    #[allow(missing_docs)]
    #[error("Downloaded sample file not seen: {0}")]
    DownloadedSampleFileNotSeen(String),

    #[allow(missing_docs)]
    #[error("Downloaded sample file has wrong hash: {0},expected: {1}, actual: {2}")]
    DownloadedSampleFileWrongHash(String, String, String),

    #[allow(missing_docs)]
    #[error("Cannot create cache directory")]
    CannotCreateCacheDir(),
}
