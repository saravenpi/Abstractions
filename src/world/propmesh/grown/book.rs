//! The book of grown looks: the mesh of every look in use, built once and named by its page, which a
//! placement carries in its variant.
//!
//! A placement keeps only a plant and a variant, so the look of a grown plant lives here and the chunk
//! builders find its mesh without knowing the planet. The book holds [`PAGES`] looks; past that it writes
//! over its oldest page, long after the chunks that read it were built.

use std::sync::{Arc, PoisonError, RwLock};

use super::{Look, build};
use crate::world::propmesh::VARIANTS;
use crate::world::propmesh::merge::Soup;

/// Pages the book holds: every variant past the shared meshes of home.
pub const PAGES: usize = 256 - VARIANTS;

/// One page: a look and its mesh.
struct Page {
    /// The look.
    look: Look,
    /// Its mesh, shared by every chunk that shows it.
    soup: Arc<Soup>,
}

/// Every page, and the one written next once the book is full.
struct Book {
    /// The pages in the order they were written.
    pages: Vec<Page>,
    /// The page written over next once all are taken.
    next: usize,
    /// How many pages were written over so far.
    edition: usize,
}

/// The one book of the process.
static BOOK: RwLock<Book> = RwLock::new(Book {
    pages: Vec::new(),
    next: 0,
    edition: 0,
});

/// Returns the placement variant of `look`, writing it into the book with its mesh when it is new.
pub fn page(look: Look) -> u8 {
    let found = |book: &Book| book.pages.iter().position(|p| p.look == look);
    let read = BOOK.read().unwrap_or_else(PoisonError::into_inner);
    if let Some(i) = found(&read) {
        return (i + VARIANTS) as u8;
    }
    drop(read);
    let soup = Arc::new(Soup::of(&build(&look).into_mesh()));
    let mut book = BOOK.write().unwrap_or_else(PoisonError::into_inner);
    let i = match found(&book) {
        Some(i) => i,
        None if book.pages.len() < PAGES => {
            book.pages.push(Page { look, soup });
            book.pages.len() - 1
        }
        None => {
            let i = book.next;
            book.pages[i] = Page { look, soup };
            book.next = (i + 1) % PAGES;
            book.edition += 1;
            i
        }
    };
    (i + VARIANTS) as u8
}

/// Returns the mesh of the look of placement variant `variant`, or nothing for a shared mesh of home.
pub(in crate::world::propmesh) fn soup(variant: u8) -> Option<Arc<Soup>> {
    let i = usize::from(variant).checked_sub(VARIANTS)?;
    let book = BOOK.read().unwrap_or_else(PoisonError::into_inner);
    book.pages.get(i).map(|p| p.soup.clone())
}

/// Returns how many triangles the look of placement variant `variant` has, 0 for one not in the book.
pub fn tris(variant: u8) -> usize {
    soup(variant).map_or(0, |s| s.positions.len() / 3)
}

/// Returns how many pages were written over so far: a variant handed out under an older edition may now
/// name another look.
pub fn edition() -> usize {
    BOOK.read().unwrap_or_else(PoisonError::into_inner).edition
}
