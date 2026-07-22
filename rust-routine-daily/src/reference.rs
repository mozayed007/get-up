pub struct ReferenceInfo {
    pub url: String,
    pub display_name: String,
}

pub fn topic_to_hello_algo_reference(tags: &[String]) -> Option<ReferenceInfo> {
    for tag in tags {
        let (url, display_name) = match tag.as_str() {
            "array" | "string" | "matrix" =>
                ("https://www.hello-algo.com/en/chapter_array_and_linkedlist/", "Array & Linked List"),
            "hash-table" =>
                ("https://www.hello-algo.com/en/chapter_hashing/", "Hash Table"),
            "linked-list" =>
                ("https://www.hello-algo.com/en/chapter_array_and_linkedlist/linked_list/", "Linked List"),
            "binary-search" =>
                ("https://www.hello-algo.com/en/chapter_searching/binary_search/", "Binary Search"),
            "divide-and-conquer" =>
                ("https://www.hello-algo.com/en/chapter_divide_and_conquer/", "Divide & Conquer"),
            "dynamic-programming" =>
                ("https://www.hello-algo.com/en/chapter_dynamic_programming/intro_to_dynamic_programming/", "Dynamic Programming"),
            "backtracking" =>
                ("https://www.hello-algo.com/en/chapter_backtracking/", "Backtracking"),
            "greedy" =>
                ("https://www.hello-algo.com/en/chapter_greedy/", "Greedy"),
            "sorting" | "counting-sort" | "bucket-sort" | "radix-sort" =>
                ("https://www.hello-algo.com/en/chapter_sorting/", "Sorting"),
            "heap" | "priority-queue" =>
                ("https://www.hello-algo.com/en/chapter_heap/", "Heap"),
            "tree" | "binary-tree" | "binary-search-tree" =>
                ("https://www.hello-algo.com/en/chapter_tree/binary_tree/", "Binary Tree"),
            "depth-first-search" =>
                ("https://www.hello-algo.com/en/chapter_tree/binary_tree_traversal/", "Binary Tree Traversal"),
            "breadth-first-search" =>
                ("https://www.hello-algo.com/en/chapter_graph/graph_traversal/", "Graph Traversal"),
            "graph" | "union-find" =>
                ("https://www.hello-algo.com/en/chapter_graph/", "Graph"),
            "stack" | "monotonic-stack" =>
                ("https://www.hello-algo.com/en/chapter_stack_and_queue/stack/", "Stack"),
            "queue" | "monotonic-queue" =>
                ("https://www.hello-algo.com/en/chapter_stack_and_queue/queue/", "Queue"),
            "recursion" =>
                ("https://www.hello-algo.com/en/chapter_computational_complexity/iteration_and_recursion/", "Recursion"),
            "trie" =>
                ("https://www.hello-algo.com/en/chapter_tree/binary_tree/", "Trie"),
            "two-pointers" =>
                ("https://www.hello-algo.com/en/chapter_array_and_linkedlist/", "Two Pointers"),
            "prefix-sum" =>
                ("https://www.hello-algo.com/en/chapter_searching/", "Prefix Sum"),
            "sliding-window" =>
                ("https://www.hello-algo.com/en/chapter_stack_and_queue/", "Sliding Window"),
            "bit-manipulation" =>
                ("https://www.hello-algo.com/en/chapter_data_structure/number_encoding/", "Bit Manipulation"),
            "design" | "iterator" =>
                ("https://www.hello-algo.com/en/chapter_array_and_linkedlist/list/", "List & Iterator"),
            _ => continue,
        };
        return Some(ReferenceInfo { url: url.to_string(), display_name: display_name.to_string() });
    }
    None
}
