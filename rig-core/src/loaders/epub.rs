use std::{
    fs::{self, File},
    path::PathBuf,
};

use super::file::FileLoaderError;
use epub::doc::{DocError, EpubDoc};
use glob::glob;
use std::io::BufReader;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubLoaderError {
    #[error("{0}")]
    FileLoaderError(#[from] FileLoaderError),
    #[error("UTF-8 conversion error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("IO error: {0}")]
    EpubError(#[from] DocError),
}

// ================================================================
// Implementing Loadable trait for loading epubs
// ================================================================

pub(crate) trait Loadable {
    fn load(self) -> Result<EpubDoc<BufReader<File>>, EpubLoaderError>;
    fn load_with_path(self) -> Result<(PathBuf, EpubDoc<BufReader<File>>), EpubLoaderError>;
}

impl Loadable for PathBuf {
    fn load(self) -> Result<EpubDoc<BufReader<File>>, EpubLoaderError> {
        EpubDoc::new(self).map_err(EpubLoaderError::EpubError)
    }
    fn load_with_path(self) -> Result<(PathBuf, EpubDoc<BufReader<File>>), EpubLoaderError> {
        let contents = EpubDoc::new(&self);
        Ok((self, contents?))
    }
}

impl<T: Loadable> Loadable for Result<T, EpubLoaderError> {
    fn load(self) -> Result<EpubDoc<BufReader<File>>, EpubLoaderError> {
        self.map(|t| t.load())?
    }
    fn load_with_path(self) -> Result<(PathBuf, EpubDoc<BufReader<File>>), EpubLoaderError> {
        self.map(|t| t.load_with_path())?
    }
}

// ================================================================
// EpubFileLoader definitions and implementations
// ================================================================

/// [EpubFileLoader] is a utility for loading pdf files from the filesystem using glob patterns or
///  directory paths. It provides methods to read file contents and handle errors gracefully.
///
/// # Errors
///
/// This module defines a custom error type [EpubLoaderError] which can represent various errors
///  that might occur during file loading operations, such as any [FileLoaderError] alongside
///  specific PDF-related errors.
///
/// # Example Usage
///
/// ```rust
/// use rig:loaders::EpubFileLoader;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a FileLoader using a glob pattern
///     let loader = EpubFileLoader::with_glob("tests/data/*.pdf")?;
///
///     // Load pdf file contents by page, ignoring any errors
///     let contents: Vec<String> = loader
///         .load_with_path()
///         .ignore_errors()
///         .by_page()
///
///     for content in contents {
///         println!("{}", content);
///     }
///
///     Ok(())
/// }
/// ```
///
/// [EpubFileLoader] uses strict typing between the iterator methods to ensure that transitions
///  between different implementations of the loaders and it's methods are handled properly by
///  the compiler.

pub struct EpubFileLoader<'a, T> {
    iterator: Box<dyn Iterator<Item = T> + 'a>,
}

impl<'a> EpubFileLoader<'a, Result<PathBuf, EpubLoaderError>> {
    /// Loads the contents of the pdfs within the iterator returned by [EpubFileLoader::with_glob]
    ///  or [EpubFileLoader::with_dir]. Loaded PDF documents are raw PDF instances that can be
    ///  further processed (by page, etc).
    ///
    /// # Example
    /// Load pdfs in directory "tests/data/*.pdf" and return the loaded documents
    ///
    /// ```rust
    /// let content = EpubFileLoader::with_glob("tests/data/*.epub")?.load().into_iter();
    /// for result in content {
    ///     match result {
    ///         Ok(doc) => println!("{}", doc),
    ///         Err(e) => eprintln!("Error reading epub: {}", e),
    ///     }
    /// }
    /// ```
    pub fn load(self) -> EpubFileLoader<'a, Result<EpubDoc<BufReader<File>>, EpubLoaderError>> {
        EpubFileLoader {
            iterator: Box::new(self.iterator.map(|res| res.load())),
        }
    }

    /// Loads the contents of the pdfs within the iterator returned by [EpubFileLoader::with_glob]
    ///  or [EpubFileLoader::with_dir]. Loaded PDF documents are raw PDF instances with their path
    ///  that can be further processed.
    ///
    /// # Example
    /// Load pdfs in directory "tests/data/*.pdf" and return the loaded documents
    ///
    /// ```rust
    /// let content = EpubFileLoader::with_glob("tests/data/*.pdf")?.load_with_path().into_iter();
    /// for result in content {
    ///     match result {
    ///         Ok((path, doc)) => println!("{:?} {}", path, doc),
    ///         Err(e) => eprintln!("Error reading pdf: {}", e),
    ///     }
    /// }
    /// ```
    pub fn load_with_path(
        self,
    ) -> EpubFileLoader<'a, Result<(PathBuf, EpubDoc<BufReader<File>>), EpubLoaderError>> {
        EpubFileLoader {
            iterator: Box::new(self.iterator.map(|res| res.load_with_path())),
        }
    }
}

// impl<'a> EpubFileLoader<'a, Result<PathBuf, EpubLoaderError>> {
//     /// Directly reads the contents of the pdfs within the iterator returned by
//     ///  [EpubFileLoader::with_glob] or [EpubFileLoader::with_dir].
//     ///
//     /// # Example
//     /// Read pdfs in directory "tests/data/*.pdf" and return the contents of the documents.
//     ///
//     /// ```rust
//     /// let content = EpubFileLoader::with_glob("tests/data/*.epub")?.read_with_path().into_iter();
//     /// for result in content {
//     ///     match result {
//     ///         Ok((path, content)) => println!("{}", content),
//     ///         Err(e) => eprintln!("Error reading pdf: {}", e),
//     ///     }
//     /// }
//     /// ```
//     pub fn read(self) -> EpubFileLoader<'a, Result<String, EpubLoaderError>> {
//         EpubFileLoader {
//             iterator: Box::new(self.iterator.map(|res| {
//                 let doc = res.load()?;
//                 Ok(doc
//                     .page_iter()
//                     .enumerate()
//                     .map(|(page_no, _)| {
//                         doc.extract_text(&[page_no as u32 + 1])
//                             .map_err(EpubLoaderError::PdfError)
//                     })
//                     .collect::<Result<Vec<String>, EpubLoaderError>>()?
//                     .into_iter()
//                     .collect::<String>())
//             })),
//         }
//     }

//     /// Directly reads the contents of the pdfs within the iterator returned by
//     ///  [EpubFileLoader::with_glob] or [EpubFileLoader::with_dir] and returns the path along with
//     ///  the content.
//     ///
//     /// # Example
//     /// Read pdfs in directory "tests/data/*.pdf" and return the content and paths of the documents.
//     ///
//     /// ```rust
//     /// let content = EpubFileLoader::with_glob("tests/data/*.pdf")?.read_with_path().into_iter();
//     /// for result in content {
//     ///     match result {
//     ///         Ok((path, content)) => println!("{:?} {}", path, content),
//     ///         Err(e) => eprintln!("Error reading pdf: {}", e),
//     ///     }
//     /// }
//     /// ```
//     pub fn read_with_path(self) -> EpubFileLoader<'a, Result<(PathBuf, String), EpubLoaderError>> {
//         EpubFileLoader {
//             iterator: Box::new(self.iterator.map(|res| {
//                 let (path, doc) = res.load_with_path()?;
//                 println!(
//                     "Loaded {:?} PDF: {:?}",
//                     path,
//                     doc.page_iter().collect::<Vec<_>>()
//                 );
//                 let content = doc
//                     .page_iter()
//                     .enumerate()
//                     .map(|(page_no, _)| {
//                         doc.extract_text(&[page_no as u32 + 1])
//                             .map_err(EpubLoaderError::PdfError)
//                     })
//                     .collect::<Result<Vec<String>, EpubLoaderError>>()?
//                     .into_iter()
//                     .collect::<String>();

//                 Ok((path, content))
//             })),
//         }
//     }
// }

// impl<'a> EpubFileLoader<'a, Document> {
//     /// Chunks the pages of a loaded document by page, flattened as a single vector.
//     ///
//     /// # Example
//     /// Load pdfs in directory "tests/data/*.pdf" and chunk all document into it's pages.
//     ///
//     /// ```rust
//     /// let content = EpubFileLoader::with_glob("tests/data/*.pdf")?.load().by_page().into_iter();
//     /// for result in content {
//     ///     match result {
//     ///         Ok(page) => println!("{}", page),
//     ///         Err(e) => eprintln!("Error reading pdf: {}", e),
//     ///     }
//     /// }
//     /// ```
//     pub fn by_page(self) -> EpubFileLoader<'a, Result<String, EpubLoaderError>> {
//         EpubFileLoader {
//             iterator: Box::new(self.iterator.flat_map(|doc| {
//                 doc.page_iter()
//                     .enumerate()
//                     .map(|(page_no, _)| {
//                         doc.extract_text(&[page_no as u32 + 1])
//                             .map_err(EpubLoaderError::PdfError)
//                     })
//                     .collect::<Vec<_>>()
//             })),
//         }
//     }
// }
