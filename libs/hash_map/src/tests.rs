#![cfg(test)]

use crate::FixedSizeHashMap;

type MyMap = FixedSizeHashMap<String, u64, 13>;
fn add_some_data(map: &mut MyMap, num: i32) {
    let keys = ["foo", "bar", "baz", "bat", "boo", "fat"];
    for (i, key) in keys.iter().enumerate() {
        if i as i32 == num {
            break;
        }
        map.insert(String::from(*key), (i as u64 +1)*100);
    }
}

#[test]
fn insert_and_get_items() {
    let mut fixed_size_map = MyMap::new();
    assert!(fixed_size_map.capacity() == 13);
    assert!(fixed_size_map.size() == 0);
    assert!(fixed_size_map.exists(&String::from("foo")) == false);
    assert!(fixed_size_map.get(&String::from("foo")) == None);
    assert!(fixed_size_map.head() == None);
    assert!(fixed_size_map.tail() == None);

    add_some_data(&mut fixed_size_map, 3);

    assert!(fixed_size_map.capacity() == 13);
    assert_eq!(fixed_size_map.size(), 3);
    assert!(fixed_size_map.exists(&String::from("foo"))
        && fixed_size_map.exists(&String::from("bar"))
        && fixed_size_map.exists(&String::from("baz"))
    );
    assert_eq!(fixed_size_map.get(&String::from("foo")), Some(&100));
    assert_eq!(fixed_size_map.get(&String::from("bar")), Some(&200));
    assert_eq!(fixed_size_map.get(&String::from("baz")), Some(&300));
    assert_eq!(fixed_size_map.head(), Some( (&String::from("baz"), &300) ));
    assert_eq!(fixed_size_map.tail(), Some( (&String::from("foo"), &100) ));

}
#[test]
fn update_items() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert_eq!(fixed_size_map.size(), 4);

    let old_val = fixed_size_map.insert(String::from("bar"), 2000);
    
    assert_eq!(fixed_size_map.size(), 4);
    assert!(fixed_size_map.get(&String::from("bar")) == Some(&2000));
    assert_eq!(old_val, Some(200));
}

#[test]
fn remove_items_from_middle() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    let old_val_of_bar = fixed_size_map.remove(&String::from("bar"));
    let old_val_of_baz = fixed_size_map.remove(&String::from("baz"));

    assert_eq!(fixed_size_map.size(), 2);
    assert_eq!(old_val_of_bar, Some(200));
    assert_eq!(old_val_of_baz, Some(300));
    assert_eq!(fixed_size_map.exists(&String::from("bar")), false);
    assert_eq!(fixed_size_map.exists(&String::from("zoo")), false);
    assert_eq!(fixed_size_map.head(), Some( (&String::from("bat"), &400) ));
    assert_eq!(fixed_size_map.tail(), Some( (&String::from("foo"), &100) ));
}

#[test]
fn remove_head_and_tail_item() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);
    
    let _ = fixed_size_map.remove(&String::from("bat"));
    let _ = fixed_size_map.remove(&String::from("foo"));

    assert_eq!(fixed_size_map.size(), 2);
    assert_eq!(fixed_size_map.head(), Some( (&String::from("baz"), &300) ));
    assert_eq!(fixed_size_map.tail(), Some( (&String::from("bar"), &200) ));
}

#[test]
fn remove_non_existent_item() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    let old_val_of_zoo = fixed_size_map.remove(&String::from("zoo"));

    assert_eq!(fixed_size_map.size(), 4);
    assert_eq!(old_val_of_zoo, None);
    assert_eq!(fixed_size_map.head(), Some( (&String::from("bat"), &400) ));
    assert_eq!(fixed_size_map.tail(), Some( (&String::from("foo"), &100) ));
}

#[test]
fn in_place_update() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    fixed_size_map.get_mut(&String::from("bar")).and_then(|v| {*v += 1000; Some(true)});
    assert_eq!(fixed_size_map.get(&String::from("bar")), Some(&1200));
}

#[test]
fn indexed_read_and_mutate() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    assert_eq!(fixed_size_map[&String::from("bar")], 200);
    fixed_size_map[&String::from("bar")] += 1000;

    assert_eq!(fixed_size_map.get(&String::from("bar")), Some(&1200));
}

#[test]
fn forward_iteration() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    let mut iter = fixed_size_map.iter_head();

    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some((&String::from("bat"), &400)) );
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some((&String::from("baz"), &300)) );
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some((&String::from("bar"), &200)) );
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some((&String::from("foo"), &100)) );
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None );
    assert_eq!(iter.next(), None ); 
}

    #[test]
fn backward_iteration() {
    let mut fixed_size_map = MyMap::new();
    add_some_data(&mut fixed_size_map, 4);
    assert!(fixed_size_map.size() == 4);

    let mut iter = fixed_size_map.iter_tail();

    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some((&String::from("foo"), &100)) );
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some((&String::from("bar"), &200)) );
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some((&String::from("baz"), &300)) );
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some((&String::from("bat"), &400)) );
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None );
    assert_eq!(iter.next(), None ); 
}