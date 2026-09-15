pub struct ReferenceInfo {
    pub url: String,
    pub display_name: String,
}

// First match wins; most specific topics first, generic array/string/matrix last.
const TOPIC_REFERENCES: &[(&[&str], &str, &str)] = &[
    (
        &["dynamic-programming"],
        "https://www.hello-algo.com/en/chapter_dynamic_programming/intro_to_dynamic_programming/",
        "Dynamic Programming",
    ),
    (
        &["graph", "union-find"],
        "https://www.hello-algo.com/en/chapter_graph/",
        "Graph",
    ),
    (
        &["trie"],
        "https://www.hello-algo.com/en/chapter_tree/binary_tree/",
        "Trie",
    ),
    (
        &["heap", "priority-queue"],
        "https://www.hello-algo.com/en/chapter_heap/",
        "Heap",
    ),
    (
        &["binary-search"],
        "https://www.hello-algo.com/en/chapter_searching/binary_search/",
        "Binary Search",
    ),
    (
        &["two-pointers"],
        "https://www.hello-algo.com/en/chapter_array_and_linkedlist/",
        "Two Pointers",
    ),
    (
        &["sliding-window"],
        "https://www.hello-algo.com/en/chapter_stack_and_queue/",
        "Sliding Window",
    ),
    (
        &["prefix-sum"],
        "https://www.hello-algo.com/en/chapter_searching/",
        "Prefix Sum",
    ),
    (
        &["backtracking"],
        "https://www.hello-algo.com/en/chapter_backtracking/",
        "Backtracking",
    ),
    (
        &["greedy"],
        "https://www.hello-algo.com/en/chapter_greedy/",
        "Greedy",
    ),
    (
        &["depth-first-search"],
        "https://www.hello-algo.com/en/chapter_tree/binary_tree_traversal/",
        "Binary Tree Traversal",
    ),
    (
        &["breadth-first-search"],
        "https://www.hello-algo.com/en/chapter_graph/graph_traversal/",
        "Graph Traversal",
    ),
    (
        &["stack", "monotonic-stack"],
        "https://www.hello-algo.com/en/chapter_stack_and_queue/stack/",
        "Stack",
    ),
    (
        &["queue", "monotonic-queue"],
        "https://www.hello-algo.com/en/chapter_stack_and_queue/queue/",
        "Queue",
    ),
    (
        &["bit-manipulation"],
        "https://www.hello-algo.com/en/chapter_data_structure/number_encoding/",
        "Bit Manipulation",
    ),
    (
        &["recursion"],
        "https://www.hello-algo.com/en/chapter_computational_complexity/iteration_and_recursion/",
        "Recursion",
    ),
    (
        &["sorting", "counting-sort", "bucket-sort", "radix-sort"],
        "https://www.hello-algo.com/en/chapter_sorting/",
        "Sorting",
    ),
    (
        &["divide-and-conquer"],
        "https://www.hello-algo.com/en/chapter_divide_and_conquer/",
        "Divide & Conquer",
    ),
    (
        &["hash-table"],
        "https://www.hello-algo.com/en/chapter_hashing/",
        "Hash Table",
    ),
    (
        &["linked-list"],
        "https://www.hello-algo.com/en/chapter_array_and_linkedlist/linked_list/",
        "Linked List",
    ),
    (
        &["tree", "binary-tree", "binary-search-tree"],
        "https://www.hello-algo.com/en/chapter_tree/binary_tree/",
        "Binary Tree",
    ),
    (
        &["design", "iterator"],
        "https://www.hello-algo.com/en/chapter_array_and_linkedlist/list/",
        "List & Iterator",
    ),
    (
        &["array", "string", "matrix"],
        "https://www.hello-algo.com/en/chapter_array_and_linkedlist/",
        "Array & Linked List",
    ),
];

pub fn topic_to_hello_algo_reference(tags: &[String]) -> Option<ReferenceInfo> {
    TOPIC_REFERENCES
        .iter()
        .find_map(|(keys, url, display_name)| {
            tags.iter()
                .any(|tag| keys.contains(&tag.as_str()))
                .then(|| ReferenceInfo {
                    url: (*url).to_string(),
                    display_name: (*display_name).to_string(),
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(tags: &[&str]) -> Option<ReferenceInfo> {
        let tags: Vec<String> = tags.iter().map(|tag| tag.to_string()).collect();
        topic_to_hello_algo_reference(&tags)
    }

    #[test]
    fn test_topic_reference_prefers_dynamic_programming_over_array() {
        let reference = resolve(&["array", "dynamic-programming"]).unwrap();
        assert_eq!(reference.url, "https://www.hello-algo.com/en/chapter_dynamic_programming/intro_to_dynamic_programming/");
        assert_eq!(reference.display_name, "Dynamic Programming");
    }

    #[test]
    fn test_topic_reference_array_fallback() {
        assert_eq!(
            resolve(&["array"]).unwrap().display_name,
            "Array & Linked List"
        );
        assert_eq!(
            resolve(&["array", "matrix"]).unwrap().display_name,
            "Array & Linked List"
        );
    }

    #[test]
    fn test_topic_reference_prefers_binary_search_tree_over_array() {
        assert_eq!(
            resolve(&["binary-search-tree", "array"])
                .unwrap()
                .display_name,
            "Binary Tree"
        );
    }

    #[test]
    fn test_topic_reference_trie() {
        assert_eq!(resolve(&["trie"]).unwrap().display_name, "Trie");
    }

    #[test]
    fn test_topic_reference_priority_queue() {
        assert_eq!(resolve(&["priority-queue"]).unwrap().display_name, "Heap");
    }

    #[test]
    fn test_topic_reference_unknown_tag() {
        assert!(resolve(&["unknown-tag"]).is_none());
    }
}
