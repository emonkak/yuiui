use std::collections::vec_deque::IntoIter;
use std::collections::VecDeque;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(pub(crate) u64);

impl Id {
    pub const ROOT: Self = Self(0);
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdPathBuf {
    path: Vec<Id>,
}

impl IdPathBuf {
    pub const fn new() -> Self {
        Self { path: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, id: Id) {
        self.path.push(id);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Id> {
        self.path.pop()
    }
}

impl Deref for IdPathBuf {
    type Target = IdPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        IdPath::from_slice(self.path.as_slice())
    }
}

impl FromIterator<Id> for IdPathBuf {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Id>,
    {
        Self {
            path: Vec::from_iter(iter),
        }
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IdPath {
    path: [Id],
}

impl IdPath {
    pub const fn from_slice(slice: &[Id]) -> &Self {
        unsafe { &*(slice as *const [Id] as *const IdPath) }
    }

    #[inline]
    pub fn top_id(&self) -> Id {
        self.path.first().copied().unwrap_or(Id::ROOT)
    }

    #[inline]
    pub fn bottom_id(&self) -> Id {
        self.path.last().copied().unwrap_or(Id::ROOT)
    }

    #[inline]
    pub fn starts_with(&self, other: &Self) -> bool {
        self.path.starts_with(&other.path)
    }

    #[inline]
    pub fn strip_init(&self, other: &Self) -> Option<&Self> {
        if self.path.starts_with(&other.path) {
            Some(Self::from_slice(&self.path[..other.path.len() + 1]))
        } else {
            None
        }
    }

    fn strip_intersection<'other>(&self, other: &'other Self) -> (&[Id], &'other [Id]) {
        let mismatched_index = match self.path.iter().zip(&other.path).position(|(x, y)| x != y) {
            None => self.path.len().min(other.path.len()),
            Some(index) => index,
        };

        let left = &self.path[mismatched_index..];
        let right = &other.path[mismatched_index..];

        (left, right)
    }
}

#[derive(Debug)]
pub struct IdSelection {
    items: VecDeque<(IdPathBuf, ComponentIndex)>,
}

impl IdSelection {
    pub fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn select(&mut self, id_path: IdPathBuf, component_index: ComponentIndex) {
        let mut should_push = true;
        self.items
            .retain_mut(|(selection_id_path, selection_component_index)| {
                match selection_id_path.strip_intersection(&id_path) {
                    ([_, ..], [_, ..]) => true,
                    ([], [_, ..]) => {
                        should_push = false;
                        true
                    }
                    ([_, ..], []) => false,
                    ([], []) => {
                        should_push = false;
                        *selection_component_index =
                            (*selection_component_index).min(component_index);
                        true
                    }
                }
            });
        if should_push {
            self.items.push_back((id_path, component_index));
        }
    }

    pub fn pop(&mut self) -> Option<(IdPathBuf, ComponentIndex)> {
        self.items.pop_front()
    }
}

impl IntoIterator for IdSelection {
    type Item = (IdPathBuf, ComponentIndex);

    type IntoIter = IntoIter<(IdPathBuf, ComponentIndex)>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub type ComponentIndex = usize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_selection() {
        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([]), 0);
        id_selection.select(IdPathBuf::from_iter([]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([]), 1);
        id_selection.select(IdPathBuf::from_iter([]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([]), 0);
        id_selection.select(IdPathBuf::from_iter([]), 1);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 1);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0), Id(1)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 1);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0), Id(1)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(2)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPathBuf::from_iter([Id(0)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPathBuf::from_iter([Id(0), Id(2)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPathBuf::from_iter([Id(0)]), 0)]
        );
    }
}
