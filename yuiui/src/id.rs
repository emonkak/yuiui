use std::collections::vec_deque::IntoIter;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(pub(crate) u64);

impl Id {
    pub const ROOT: Self = Self(0);
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IdPath {
    path: Vec<Id>,
}

impl IdPath {
    pub const fn new() -> Self {
        Self { path: Vec::new() }
    }

    pub fn top_id(&self) -> Id {
        self.path.first().copied().unwrap_or(Id::ROOT)
    }

    pub fn bottom_id(&self) -> Id {
        self.path.last().copied().unwrap_or(Id::ROOT)
    }

    pub fn starts_with(&self, needle: &Self) -> bool {
        self.path.starts_with(&needle.path)
    }

    pub fn push(&mut self, id: Id) {
        self.path.push(id);
    }

    pub fn pop(&mut self) -> Id {
        self.path.pop().unwrap()
    }

    fn strip_intersection<'other>(&self, other: &'other Self) -> (&[Id], &'other [Id]) {
        let mismatched_index = match self.path.iter().zip(&other.path).position(|(x, y)| x != y) {
            None => self.path.len().min(other.path.len()),
            Some(diff) => diff,
        };

        let left = &self.path[mismatched_index..];
        let right = &other.path[mismatched_index..];

        (left, right)
    }
}

impl FromIterator<Id> for IdPath {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Id>,
    {
        Self {
            path: Vec::from_iter(iter),
        }
    }
}

#[derive(Debug)]
pub struct IdSelection {
    items: VecDeque<(IdPath, ComponentIndex)>,
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

    pub fn select(&mut self, id_path: IdPath, component_index: ComponentIndex) {
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

    pub fn pop(&mut self) -> Option<(IdPath, ComponentIndex)> {
        self.items.pop_front()
    }
}

impl IntoIterator for IdSelection {
    type Item = (IdPath, ComponentIndex);

    type IntoIter = IntoIter<(IdPath, ComponentIndex)>;

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
        id_selection.select(IdPath::from_iter([]), 0);
        id_selection.select(IdPath::from_iter([]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([]), 1);
        id_selection.select(IdPath::from_iter([]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([]), 0);
        id_selection.select(IdPath::from_iter([]), 1);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0)]), 0);
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPath::from_iter([Id(0)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 1);
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0), Id(1)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 1);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0), Id(1)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPath::from_iter([Id(0), Id(2)]), 0);
        id_selection.select(IdPath::from_iter([Id(0)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0)]), 0)]
        );

        let mut id_selection = IdSelection::new();
        id_selection.select(IdPath::from_iter([Id(0)]), 0);
        id_selection.select(IdPath::from_iter([Id(0), Id(1)]), 0);
        id_selection.select(IdPath::from_iter([Id(0), Id(2)]), 0);
        assert_eq!(
            id_selection.into_iter().collect::<Vec<_>>(),
            vec![(IdPath::from_iter([Id(0)]), 0)]
        );
    }
}
