mod array;
mod either;
mod hlist;
mod option;
mod vec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RenderStatus {
    Unchanged,
    Changed,
    Swapped,
}
