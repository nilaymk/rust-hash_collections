#![cfg(test)]

use std::vec;

use crate::FixedSizeHashGraphMap;

type MyGraph = FixedSizeHashGraphMap<String, (), 97>;

#[test]
fn insert_edges_once() {
    let mut graph = MyGraph::new();
    let _ = graph.insert(
        ("foo".to_string(), ()),
        vec![("bar".to_string(), ()), ("baz".to_string(), ())],
    );

    assert_eq!(
        graph
            .node(&"foo".to_string())
            .map_or(0, |node| node.out_edge_weight(&"bar".to_string())),
        1
    );
    assert_eq!(
        graph
            .node(&"foo".to_string())
            .map_or(0, |node| node.out_edge_weight(&"baz".to_string())),
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
        ],
    );

    assert_eq!(
        graph
            .node(&"foo".to_string())
            .map_or(0, |node| node.out_edge_weight(&"bar".to_string())),
        2
    );
    assert_eq!(
        graph
            .node(&"foo".to_string())
            .map_or(0, |node| node.out_edge_weight(&"baz".to_string())),
        1
    );
}

#[test]
fn multiple_nodes_and_edges() {
    let mut graph =  FixedSizeHashGraphMap::<String, u64, 97>::new();

    let _ = graph.insert(
        ("foo".to_string(), 100),
        vec![
            ("bar".to_string(), 200),
            ("baz".to_string(), 300)
        ]
    );
    let _ = graph.insert(
        ("bat".to_string(), 400),
        vec![
            ("boo".to_string(), 500),
            ("foo".to_string(), 1000),
        ]
    );
    graph.connect_to(
        &"boo".to_string(),
        vec![
            &"baz".to_string(),
            &"foo".to_string(),
        ]
    );
    graph.connect_to(
        &"foo".to_string(),
        vec![
            &"bar".to_string(),
            &"bar".to_string(),
        ]
    );

    assert_eq!(graph.out_edge_weight(&"foo".to_string(), &"bar".to_string()), 3);
    assert_eq!(graph.out_edge_weight(&"foo".to_string(), &"baz".to_string()), 1);
    assert_eq!(graph.out_edge_weight(&"bat".to_string(), &"boo".to_string()), 1);
    assert_eq!(graph.out_edge_weight(&"bat".to_string(), &"foo".to_string()), 1);
    assert_eq!(graph.out_edge_weight(&"boo".to_string(), &"foo".to_string()), 1);
    assert_eq!(graph.out_edge_weight(&"boo".to_string(), &"baz".to_string()), 1);
    assert_eq!(graph.out_edge_weight(&"bat".to_string(), &"bar".to_string()), 0);
    assert_eq!(graph.out_edge_weight(&"bat".to_string(), &"baz".to_string()), 0);
}
