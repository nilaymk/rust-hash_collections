#![cfg(test)]

use crate::FixedSizeHashSet;

type MySet = FixedSizeHashSet<String, 13>;

fn add_some_data(set: &mut MySet, num: usize) {
    let keys = ["foo", "bar", "baz", "bat", "boo", "fat"];

    keys.iter()
        .enumerate()
        .filter(|(i, _)| *i < num)
        .for_each(
            |(_, k)| {
                assert!(set.insert(k.to_string()).is_ok());
            }
        );
}

#[test]
fn insert_and_get_items() {
    let mut fixed_size_set = MySet::new();
    assert!(fixed_size_set.capacity() == 13);
    assert!(fixed_size_set.size() == 0);
    assert!(fixed_size_set.exists(&String::from("foo")) == false);
    assert!(fixed_size_set.head() == None);
    assert!(fixed_size_set.tail() == None);

    add_some_data(&mut fixed_size_set, 3);

    assert!(fixed_size_set.capacity() == 13);
    assert_eq!(fixed_size_set.size(), 3);
    assert!(
        fixed_size_set.exists(&String::from("foo"))
            && fixed_size_set.exists(&String::from("bar"))
            && fixed_size_set.exists(&String::from("baz"))
    );

    assert_eq!(fixed_size_set.tail(), Some(&String::from("baz")));
    assert_eq!(fixed_size_set.head(), Some(&String::from("foo")));
}

#[test]
fn remove_items_from_middle() {
    let mut fixed_size_set = MySet::new();
    add_some_data(&mut fixed_size_set, 4);
    assert!(fixed_size_set.size() == 4);

    assert!(fixed_size_set.remove(&String::from("bar")));
    assert!(fixed_size_set.remove(&String::from("baz")));

    assert_eq!(fixed_size_set.size(), 2);
    assert_eq!(fixed_size_set.exists(&String::from("bar")), false);
    assert_eq!(fixed_size_set.exists(&String::from("zoo")), false);
    assert_eq!(fixed_size_set.tail(), Some(&String::from("bat")));
    assert_eq!(fixed_size_set.head(), Some(&String::from("foo")));
}

#[test]
fn remove_head_and_tail_item() {
    let mut fixed_size_set = MySet::new();
    add_some_data(&mut fixed_size_set, 4);
    assert!(fixed_size_set.size() == 4);

    assert!(fixed_size_set.remove(&String::from("bat")));
    assert!(fixed_size_set.remove(&String::from("foo")));

    assert_eq!(fixed_size_set.size(), 2);
    assert_eq!(fixed_size_set.tail(), Some(&String::from("baz")));
    assert_eq!(fixed_size_set.head(), Some(&String::from("bar")));
}

#[test]
fn remove_non_existent_item() {
    let mut fixed_size_set = MySet::new();
    add_some_data(&mut fixed_size_set, 4);
    assert!(fixed_size_set.size() == 4);

    assert_eq!(fixed_size_set.remove(&String::from("zoo")), false);

    assert_eq!(fixed_size_set.size(), 4);
    assert_eq!(fixed_size_set.tail(), Some(&String::from("bat")));
    assert_eq!(fixed_size_set.head(), Some(&String::from("foo")));
}

#[test]
fn backward_iteration() {
    let mut fixed_size_set = MySet::new();
    add_some_data(&mut fixed_size_set, 4);
    assert!(fixed_size_set.size() == 4);

    let mut iter = fixed_size_set.iter_tail();

    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&String::from("bat")));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&String::from("baz")));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some(&String::from("bar")));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&String::from("foo")));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn forward_iteration() {
    let mut fixed_size_set = MySet::new();
    add_some_data(&mut fixed_size_set, 4);
    assert!(fixed_size_set.size() == 4);

    let mut iter = fixed_size_set.iter_head();

    assert_eq!(iter.size_hint(), (4, Some(4)));
    assert_eq!(iter.next(), Some(&String::from("foo")));
    assert_eq!(iter.size_hint(), (3, Some(3)));
    assert_eq!(iter.next(), Some(&String::from("bar")));
    assert_eq!(iter.size_hint(), (2, Some(2)));
    assert_eq!(iter.next(), Some(&String::from("baz")));
    assert_eq!(iter.size_hint(), (1, Some(1)));
    assert_eq!(iter.next(), Some(&String::from("bat")));
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn insert_same_item_multiple_times() {
    let mut myset = FixedSizeHashSet::<u64, 13>::new();

    assert!(myset.insert(5).is_ok_and(|r| r==true));
    assert!(myset.insert(5).is_ok_and(|r| r==false));
    assert!(myset.insert(5).is_ok_and(|r| r==false));

    assert_eq!(myset.size(), 1)
}
