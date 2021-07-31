use super::*;

#[test]
fn test_is_attached() {
    let mut tree = Tree::new();
    assert!(!tree.contains(0));

    let root = tree.attach("root");
    assert!(tree.contains(root));
}

#[test]
fn test_is_root() {
    let mut tree = Tree::new();

    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");

    assert_eq!(tree[root].is_root(), true);
    assert_eq!(tree[foo].is_root(), false);
    assert_eq!(tree[bar].is_root(), false);
}

#[test]
fn test_append_child() {
    let mut tree = Tree::new();
    let root = tree.attach("root");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );

    let foo = tree.append_child(root, "foo");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        }
    );

    let bar = tree.append_child(root, "bar");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(bar),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[bar],
        Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        }
    );
}

#[test]
fn test_prepend_child() {
    let mut tree = Tree::new();
    let root = tree.attach("root");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );

    let foo = tree.prepend_child(root, "foo");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        }
    );

    let bar = tree.prepend_child(root, "bar");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(bar),
                last_child: Some(foo),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(bar),
            next_sibling: None,
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[bar],
        Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(foo),
            parent: Some(root),
        }
    );
}

#[test]
fn test_insert_before() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");
    let baz = tree.insert_before(foo, "baz");
    let qux = tree.insert_before(foo, "qux");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(baz),
                last_child: Some(bar),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(qux),
            next_sibling: Some(bar),
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[bar],
        Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: None,
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[baz],
        Link {
            current: Node {
                data: "baz",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(qux),
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[qux],
        Link {
            current: Node {
                data: "qux",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(baz),
            next_sibling: Some(foo),
            parent: Some(root),
        }
    );
}

#[should_panic]
#[test]
fn test_insert_before_should_panic() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    tree.insert_before(root, "foo");
}

#[test]
fn test_insert_after() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");
    let baz = tree.insert_after(bar, "baz");
    let qux = tree.insert_after(bar, "qux");

    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(foo),
                last_child: Some(baz),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[foo],
        Link {
            current: Node {
                data: "foo",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: Some(bar),
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[bar],
        Link {
            current: Node {
                data: "bar",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(foo),
            next_sibling: Some(qux),
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[baz],
        Link {
            current: Node {
                data: "baz",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(qux),
            next_sibling: None,
            parent: Some(root),
        }
    );
    assert_eq!(
        tree[qux],
        Link {
            current: Node {
                data: "qux",
                first_child: None,
                last_child: None,
            },
            prev_sibling: Some(bar),
            next_sibling: Some(baz),
            parent: Some(root),
        }
    );
}

#[should_panic]
#[test]
fn test_insert_after_should_panic() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    tree.insert_after(root, "foo");
}

#[test]
fn test_split_subtree() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "quux");

    let subtree = tree.split_subtree(foo);

    assert!(!subtree.contains(root));
    assert_eq!(&subtree[foo], &tree[foo]);
    assert_eq!(&subtree[bar], &tree[bar]);
    assert_eq!(&subtree[baz], &tree[baz]);
    assert_eq!(&subtree[qux], &tree[qux]);
    assert!(!subtree.contains(quux));
}

#[test]
fn test_detach() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "quux");

    let (link, subtree) = tree.detach(foo);
    assert_eq!(
        link,
        Link {
            current: Node {
                data: "foo",
                first_child: Some(bar),
                last_child: Some(qux),
            },
            prev_sibling: None,
            next_sibling: Some(quux),
            parent: Some(root),
        }
    );
    assert_eq!(
        subtree.collect::<Vec<_>>(),
        [
            (
                baz,
                Link {
                    current: Node {
                        data: "baz",
                        first_child: None,
                        last_child: None,
                    },
                    prev_sibling: None,
                    next_sibling: None,
                    parent: Some(bar),
                }
            ),
            (
                bar,
                Link {
                    current: Node {
                        data: "bar",
                        first_child: Some(baz),
                        last_child: Some(baz),
                    },
                    prev_sibling: None,
                    next_sibling: Some(qux),
                    parent: Some(foo),
                }
            ),
            (
                qux,
                Link {
                    current: Node {
                        data: "qux",
                        first_child: None,
                        last_child: None,
                    },
                    prev_sibling: Some(bar),
                    next_sibling: None,
                    parent: Some(foo),
                }
            ),
        ]
    );
    assert_eq!(
        tree[root],
        Link {
            current: Node {
                data: "root",
                first_child: Some(quux),
                last_child: Some(quux),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        tree[quux],
        Link {
            current: Node {
                data: "quux",
                first_child: None,
                last_child: None,
            },
            prev_sibling: None,
            next_sibling: None,
            parent: Some(root),
        }
    );
    assert!(!tree.contains(foo));
    assert!(!tree.contains(bar));
    assert!(!tree.contains(baz));
    assert!(!tree.contains(qux));

    let (link, subtree) = tree.detach(root);
    assert_eq!(
        link,
        Link {
            current: Node {
                data: "root",
                first_child: Some(quux),
                last_child: Some(quux),
            },
            prev_sibling: None,
            next_sibling: None,
            parent: None,
        }
    );
    assert_eq!(
        subtree.collect::<Vec<_>>(),
        [(
            quux,
            Link {
                current: Node {
                    data: "quux",
                    first_child: None,
                    last_child: None,
                },
                prev_sibling: None,
                next_sibling: None,
                parent: Some(root),
            }
        ),]
    );
    assert!(!tree.contains(root));
    assert!(!tree.contains(foo));
    assert!(!tree.contains(bar));
    assert!(!tree.contains(baz));
    assert!(!tree.contains(qux));
    assert!(!tree.contains(quux));
}

#[test]
fn test_ancestors() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");
    let baz = tree.append_child(foo, "baz");
    let qux = tree.append_child(baz, "qux");
    let quux = tree.append_child(qux, "quux");
    let corge = tree.append_child(baz, "corge");

    assert_eq!(tree.ancestors(root).collect::<Vec<_>>(), []);
    assert_eq!(
        tree.ancestors(foo).collect::<Vec<_>>(),
        [(root, &tree[root])]
    );
    assert_eq!(
        tree.ancestors(bar).collect::<Vec<_>>(),
        [(root, &tree[root])]
    );
    assert_eq!(
        tree.ancestors(baz).collect::<Vec<_>>(),
        [(foo, &tree[foo]), (root, &tree[root])]
    );
    assert_eq!(
        tree.ancestors(qux).collect::<Vec<_>>(),
        [(baz, &tree[baz]), (foo, &tree[foo]), (root, &tree[root])]
    );
    assert_eq!(
        tree.ancestors(quux).collect::<Vec<_>>(),
        [
            (qux, &tree[qux]),
            (baz, &tree[baz]),
            (foo, &tree[foo]),
            (root, &tree[root])
        ]
    );
    assert_eq!(
        tree.ancestors(corge).collect::<Vec<_>>(),
        [(baz, &tree[baz]), (foo, &tree[foo]), (root, &tree[root])]
    );

    for node_id in &[root, foo, bar, baz, qux, quux, corge] {
        assert_eq!(
            tree.ancestors(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.ancestors_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_children() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");
    let baz = tree.append_child(foo, "baz");
    let qux = tree.append_child(baz, "qux");
    let quux = tree.append_child(qux, "quux");
    let corge = tree.append_child(baz, "corge");

    assert_eq!(
        tree.children(root).collect::<Vec<_>>(),
        [(foo, &tree[foo]), (bar, &tree[bar])]
    );
    assert_eq!(tree.children(foo).collect::<Vec<_>>(), [(baz, &tree[baz])]);
    assert_eq!(tree.children(bar).collect::<Vec<_>>(), []);
    assert_eq!(
        tree.children(baz).collect::<Vec<_>>(),
        [(qux, &tree[qux]), (corge, &tree[corge])]
    );
    assert_eq!(
        tree.children(qux).collect::<Vec<_>>(),
        [(quux, &tree[quux])]
    );
    assert_eq!(tree.children(quux).collect::<Vec<_>>(), []);
    assert_eq!(tree.children(corge).collect::<Vec<_>>(), []);

    for node_id in &[root, foo, bar, baz, qux, quux, corge] {
        assert_eq!(
            tree.children(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.children_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_siblings() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(root, "bar");
    let baz = tree.append_child(root, "baz");

    assert_eq!(tree.prev_siblings(root).collect::<Vec<_>>(), []);
    assert_eq!(tree.prev_siblings(foo).collect::<Vec<_>>(), []);
    assert_eq!(
        tree.prev_siblings(bar).collect::<Vec<_>>(),
        [(foo, &tree[foo])]
    );
    assert_eq!(
        tree.prev_siblings(baz).collect::<Vec<_>>(),
        [(bar, &tree[bar]), (foo, &tree[foo])]
    );

    assert_eq!(tree.next_siblings(root).collect::<Vec<_>>(), []);
    assert_eq!(
        tree.next_siblings(foo).collect::<Vec<_>>(),
        [(bar, &tree[bar]), (baz, &tree[baz])]
    );
    assert_eq!(
        tree.next_siblings(bar).collect::<Vec<_>>(),
        [(baz, &tree[baz])]
    );
    assert_eq!(tree.next_siblings(baz).collect::<Vec<_>>(), []);

    for node_id in &[root, foo, bar, baz] {
        assert_eq!(
            tree.prev_siblings(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.prev_siblings_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            tree.next_siblings(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.next_siblings_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_pre_ordered_descendants() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "qux");

    assert_eq!(
        tree.pre_ordered_descendants(root).collect::<Vec<_>>(),
        &[
            (foo, &tree[foo]),
            (bar, &tree[bar]),
            (baz, &tree[baz]),
            (qux, &tree[qux]),
            (quux, &tree[quux])
        ]
    );
    assert_eq!(
        tree.pre_ordered_descendants(foo).collect::<Vec<_>>(),
        &[(bar, &tree[bar]), (baz, &tree[baz]), (qux, &tree[qux])]
    );
    assert_eq!(
        tree.pre_ordered_descendants(bar).collect::<Vec<_>>(),
        &[(baz, &tree[baz])]
    );
    assert_eq!(tree.pre_ordered_descendants(baz).collect::<Vec<_>>(), &[]);
    assert_eq!(tree.pre_ordered_descendants(qux).collect::<Vec<_>>(), &[]);
    assert_eq!(tree.pre_ordered_descendants(quux).collect::<Vec<_>>(), &[]);

    for node_id in &[root, foo, bar, baz, qux, quux] {
        assert_eq!(
            tree.pre_ordered_descendants(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.pre_ordered_descendants_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_post_ordered_descendants() {
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "qux");

    assert_eq!(
        tree.post_ordered_descendants(root).collect::<Vec<_>>(),
        &[
            (baz, &tree[baz]),
            (bar, &tree[bar]),
            (qux, &tree[qux]),
            (foo, &tree[foo]),
            (quux, &tree[quux])
        ]
    );
    assert_eq!(
        tree.post_ordered_descendants(foo).collect::<Vec<_>>(),
        &[(baz, &tree[baz]), (bar, &tree[bar]), (qux, &tree[qux])]
    );
    assert_eq!(
        tree.post_ordered_descendants(bar).collect::<Vec<_>>(),
        &[(baz, &tree[baz])]
    );
    assert_eq!(tree.post_ordered_descendants(baz).collect::<Vec<_>>(), &[]);
    assert_eq!(tree.post_ordered_descendants(qux).collect::<Vec<_>>(), &[]);
    assert_eq!(tree.post_ordered_descendants(quux).collect::<Vec<_>>(), &[]);

    for node_id in &[root, foo, bar, baz, qux, quux] {
        assert_eq!(
            tree.post_ordered_descendants(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>(),
            tree.post_ordered_descendants_mut(*node_id)
                .map(|(index, link)| (index, link as *const _))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_walk() {
    //           root
    //          /   \
    //       foo    quux
    //      /   \
    //   bar    qux
    //   /
    // baz
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "quux");

    assert_eq!(
        tree.walk(root).collect::<Vec<_>>(),
        &[
            (root, &tree[root], WalkDirection::Downward),
            (foo, &tree[foo], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
            (quux, &tree[quux], WalkDirection::Sideward),
            (root, &tree[root], WalkDirection::Upward),
        ]
    );
    assert_eq!(
        tree.walk(foo).collect::<Vec<_>>(),
        &[
            (foo, &tree[foo], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
            (qux, &tree[qux], WalkDirection::Sideward),
            (foo, &tree[foo], WalkDirection::Upward),
        ]
    );
    assert_eq!(
        tree.walk(bar).collect::<Vec<_>>(),
        &[
            (bar, &tree[bar], WalkDirection::Downward),
            (baz, &tree[baz], WalkDirection::Downward),
            (bar, &tree[bar], WalkDirection::Upward),
        ]
    );
    assert_eq!(
        tree.walk(baz).collect::<Vec<_>>(),
        &[(baz, &tree[baz], WalkDirection::Downward),]
    );
    assert_eq!(
        tree.walk(qux).collect::<Vec<_>>(),
        &[(qux, &tree[qux], WalkDirection::Downward),]
    );
    assert_eq!(
        tree.walk(quux).collect::<Vec<_>>(),
        &[(quux, &tree[quux], WalkDirection::Downward),]
    );

    for node_id in &[root, foo, bar, baz, qux, quux] {
        assert_eq!(
            tree.walk(*node_id)
                .map(|(index, link, direction)| (index, link as *const _, direction))
                .collect::<Vec<_>>(),
            tree.walk_mut(*node_id)
                .map(|(index, link, direction)| (index, link as *const _, direction))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_walk_next_if() {
    //           root
    //          /   \
    //       foo    quux
    //      /   \
    //   bar    qux
    //   /
    // baz
    let mut tree = Tree::new();
    let root = tree.attach("root");
    let foo = tree.append_child(root, "foo");
    let bar = tree.append_child(foo, "bar");
    let baz = tree.append_child(bar, "baz");
    let qux = tree.append_child(foo, "qux");
    let quux = tree.append_child(root, "quux");

    let mut walker = tree.walk(root);
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((root, &tree[root], WalkDirection::Downward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((foo, &tree[foo], WalkDirection::Downward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((qux, &tree[qux], WalkDirection::Sideward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((foo, &tree[foo], WalkDirection::Upward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((quux, &tree[quux], WalkDirection::Sideward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((root, &tree[root], WalkDirection::Upward))
    );
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    let mut walker = tree.walk(foo);
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((foo, &tree[foo], WalkDirection::Downward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((qux, &tree[qux], WalkDirection::Sideward))
    );
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((foo, &tree[foo], WalkDirection::Upward))
    );
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    let mut walker = tree.walk(bar);
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    let mut walker = tree.walk(baz);
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((baz, &tree[baz], WalkDirection::Downward))
    );
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    let mut walker = tree.walk(qux);
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((qux, &tree[qux], WalkDirection::Downward))
    );
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    let mut walker = tree.walk(quux);
    assert_eq!(
        walker.next_if(|id, _| id != bar),
        Some((quux, &tree[quux], WalkDirection::Downward))
    );
    assert_eq!(walker.next_if(|id, _| id != bar), None);

    for &node_id in &[root, foo, bar, baz, qux, quux] {
        let mut xs = Vec::new();
        let mut walker = tree.walk(node_id);

        while let Some((index, link, direction)) = walker.next_if(|id, _| id != bar) {
            xs.push((index, link as *const _, direction));
        }

        let mut ys = Vec::new();
        let mut walker = tree.walk_mut(node_id);

        while let Some((index, link, direction)) = walker.next_if(|id, _| id != bar) {
            ys.push((index, link as *const _, direction));
        }

        assert_eq!(xs, ys);
    }
}
