use std::collections::vec_deque::IntoIter;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(pub(crate) u64);

impl Id {
    pub const ROOT: Self = Self(0);
}

pub type IdPathBuf = Vec<Id>;

pub type IdPath = [Id];

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
            .retain_mut(
                |(selection_id_path, selection_component_index)| match strip_intersection(
                    &selection_id_path,
                    &id_path,
                ) {
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
                },
            );
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

fn strip_intersection<'a, 'b>(left: &'a IdPath, right: &'b IdPath) -> (&'a [Id], &'b [Id]) {
    let mismatched_index = match left.iter().zip(right).position(|(x, y)| x != y) {
        None => left.len().min(right.len()),
        Some(index) => index,
    };

    let left = &left[mismatched_index..];
    let right = &right[mismatched_index..];

    (left, right)
}

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
