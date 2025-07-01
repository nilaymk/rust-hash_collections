struct Node {
    value: String,
    connected_to: Vec<usize>,
    connected_from: Vec<usize>,
}

struct Graph {
    _nodes: Vec<Node>,
}

impl Graph {
    fn _insert_or_get(&mut self, value: String) -> usize {
        match self._nodes.iter().position(|node| *node.value == *value) {
            Some(i) => i,
            None => {
                self._nodes.push(Node {
                    value,
                    connected_to: Vec::new(),
                    connected_from: Vec::new(),
                });
                let i = self._nodes.len() - 1;
                i
            }
        }
    }

    fn add_nodes_and_connect(&mut self, value: String, connect_to: Vec<String>) {
        let index = self._insert_or_get(value);
        for target_value in connect_to {
            let target_index = self._insert_or_get(target_value);
            if index != target_index {
                if self._nodes[index]
                    .connected_to
                    .iter()
                    .any(|i| *i == target_index)
                    == false
                {
                    self._nodes[index].connected_to.push(target_index);
                }
                if self._nodes[target_index]
                    .connected_from
                    .iter()
                    .any(|i| *i == index)
                    == false
                {
                    self._nodes[target_index].connected_from.push(index);
                }
            }
        }
    }
}

fn main() {
    let mut graph = Graph { _nodes: Vec::new() };
    // foo --> bar <--+
    //  ^  \          |
    //  |   +->  baz -+
    //  +         \
    //   \         +-> boo
    //    \             ^
    //     |            |
    //    bat ----------+
    //

    graph.add_nodes_and_connect(
        "foo".to_string(),
        vec!["bar".to_string(), "baz".to_string()],
    );
    graph.add_nodes_and_connect(
        "baz".to_string(),
        vec!["bar".to_string(), "boo".to_string()],
    );
    graph.add_nodes_and_connect(
        "bat".to_string(),
        vec!["boo".to_string(), "foo".to_string()],
    );

    println!("****************GRAPH**************");

    for node in graph._nodes.iter() {
        println!("Node '{}':", node.value);
        println!("\t connected to:");
        for connected_to in node.connected_to.iter() {
            println!("\t\t Node '{}':", graph._nodes[*connected_to].value);
        }
    }
}
