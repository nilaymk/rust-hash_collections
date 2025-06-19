#![cfg(test)]
use std::hash::Hasher;

use crate::hash_map::FixedSizeHashMapImpl;

struct HighCollisionHasher {}

impl Hasher for HighCollisionHasher {
    fn finish(&self) -> u64 {
        3
    }

    fn write(&mut self, _: &[u8]) {}
}

impl Default for HighCollisionHasher {
    fn default() -> Self {
        HighCollisionHasher {}
    }
}

type MyHighCollisionMap = FixedSizeHashMapImpl<String, String, 7, HighCollisionHasher>;

fn add_some_data(map: &mut MyHighCollisionMap, num: i32) {
    let keys = ["foo", "bar", "baz", "bat", "boo", "fat", "qux"];
    for (i, key) in keys.iter().enumerate() {
        if i as i32 == num {
            break;
        }
        map.insert(String::from(*key), ((i as u64 + 1) * 100).to_string());
    }
}

#[test]
fn insert_and_get_colliding_items() {
    let mut high_collision_map = MyHighCollisionMap::new();

    add_some_data(&mut high_collision_map, 3);

    assert_eq!(high_collision_map.size(), 3);
    assert!(
        high_collision_map.exists(&String::from("foo"))
            && high_collision_map.exists(&String::from("bar"))
            && high_collision_map.exists(&String::from("baz"))
    );
    assert_eq!(
        high_collision_map.get(&String::from("foo")),
        Some(&"100".to_string())
    );
    assert_eq!(
        high_collision_map.get(&String::from("bar")),
        Some(&"200".to_string())
    );
    assert_eq!(
        high_collision_map.get(&String::from("baz")),
        Some(&"300".to_string())
    );
    assert_eq!(
        high_collision_map.head(),
        Some((&String::from("baz"), &"300".to_string()))
    );
    assert_eq!(
        high_collision_map.tail(),
        Some((&String::from("foo"), &"100".to_string()))
    );
}

#[test]
fn update_items() {
    let mut high_collision_map = MyHighCollisionMap::new();
    add_some_data(&mut high_collision_map, 4);
    assert_eq!(high_collision_map.size(), 4);

    let old_val = high_collision_map.insert(String::from("bar"), String::from("2000"));

    assert_eq!(high_collision_map.size(), 4);
    assert!(high_collision_map.get(&String::from("bar")) == Some(&"2000".to_string()));
    assert_eq!(old_val, Some("200".to_string()));
}

#[test]
fn remove_items_from_middle() {
    let mut high_collision_map = MyHighCollisionMap::new();
    add_some_data(&mut high_collision_map, 5);
    assert!(high_collision_map.size() == 5);

    let old_val_of_bar = high_collision_map.remove(&String::from("baz"));
    let old_val_of_baz = high_collision_map.remove(&String::from("bat"));

    assert_eq!(high_collision_map.size(), 3);
    assert_eq!(old_val_of_bar, Some("300".to_string()));
    assert_eq!(old_val_of_baz, Some("400".to_string()));
    assert_eq!(high_collision_map.exists(&String::from("baz")), false);
    assert_eq!(high_collision_map.exists(&String::from("bat")), false);
    assert_eq!(high_collision_map[&String::from("foo")], "100".to_string());
    assert_eq!(high_collision_map[&String::from("bar")], "200".to_string());
    assert_eq!(high_collision_map[&String::from("boo")], "500".to_string());
}
