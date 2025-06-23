type MyMap = hash_collections::FixedSizeHashMap<String, String, 97>;

fn main() {
    let mut fmap: MyMap = MyMap::new();
    let _ = fmap.insert(String::from("Hello"), String::from("1234"));

    println!(
        "Fmap Size: {}, Capacity {}, value of \"Hello\": {}",
        fmap.size(),
        fmap.capacity(),
        fmap.get(&String::from("Hello")).unwrap()
    );
}
