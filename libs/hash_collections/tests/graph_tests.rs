#![cfg(test)]

use hash_collections::FixedSizeHashGraphMap;

type MyGraph = FixedSizeHashGraphMap<String, (), 97>;

#[test]
fn insert_edges_once() {
    let mut graph = MyGraph::new();
    let _ = graph.insert(
        ("foo".to_string(), ()),
        vec![
            ("bar".to_string(), ()),
            ("baz".to_string(), ())
        ]
    );

    assert_eq!(
        graph.node(&"foo".to_string()).map_or(0, |node| node.out_edge_weight(&"bar".to_string())),
        1
    );
    assert_eq!(
        graph.node(&"foo".to_string()).map_or(0, |node| node.out_edge_weight(&"baz".to_string())),
        1
    );
}

#[test]
fn insert_edges_multiple_times() {
    let mut graph = MyGraph::new();
    let _ = graph.insert(
        ("foo".to_string(), ()),
        vec![
            ("bar".to_string(), ()),
            ("baz".to_string(), ()),
            ("bar".to_string(), ()),
        ]
    );

    assert_eq!(
        graph.node(&"foo".to_string()).map_or(0, |node| node.out_edge_weight(&"bar".to_string())),
        2
    );
    assert_eq!(
        graph.node(&"foo".to_string()).map_or(0, |node| node.out_edge_weight(&"baz".to_string())),
        1
    );
}